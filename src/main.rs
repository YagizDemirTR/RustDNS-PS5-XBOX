use hickory_proto::op::{Message, MessageType, OpCode, ResponseCode};
use hickory_proto::rr::{RData, Record, RecordType};
use hickory_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use hickory_resolver::TokioAsyncResolver;
use local_ip_address::local_ip;
use once_cell::sync::Lazy;
use std::error::Error;
use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::sync::Arc;
use tokio::io::{copy_bidirectional, AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream, UdpSocket};

static UPSTREAM_RESOLVER: Lazy<Arc<TokioAsyncResolver>> = Lazy::new(|| {
    let mut config = ResolverConfig::cloudflare();
    config.add_name_server(NameServerConfig {
        socket_addr: "8.8.8.8:53".parse().unwrap(),
        protocol: Protocol::Udp,
        tls_dns_name: None,
        trust_negative_responses: false,
        bind_addr: None,
    });
    let opts = ResolverOpts::default();
    Arc::new(TokioAsyncResolver::tokio(config, opts))
});

fn is_blocked_domain(name: &str) -> bool {
    let lower = name.to_lowercase();
    let trimmed = lower.trim_end_matches('.');
    trimmed.ends_with("roblox.com")
        || trimmed.ends_with("rbxcdn.com")
        || trimmed.ends_with("robloxext.com")
        || trimmed.ends_with("rbx.com")
        || trimmed.ends_with("robloxlabs.com")
        || trimmed.ends_with("discord.com")
        || trimmed.ends_with("discord.gg")
        || trimmed.ends_with("discordapp.com")
        || trimmed.ends_with("discordapp.net")
        || trimmed.ends_with("discordstatus.com")
        || trimmed.ends_with("discord.media")
        || trimmed.ends_with("discordcdn.com")
}

fn extract_sni(buf: &[u8]) -> Option<String> {
    if buf.len() < 5 || buf[0] != 0x16 {
        return None;
    }
    let mut pos = 5;
    if pos + 4 > buf.len() || buf[pos] != 0x01 {
        return None;
    }
    pos += 4;
    pos += 34;
    if pos >= buf.len() {
        return None;
    }
    let sid_len = buf[pos] as usize;
    pos += 1 + sid_len;
    if pos + 2 > buf.len() {
        return None;
    }
    let cs_len = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
    pos += 2 + cs_len;
    if pos + 1 > buf.len() {
        return None;
    }
    let comp_len = buf[pos] as usize;
    pos += 1 + comp_len;
    if pos + 2 > buf.len() {
        return None;
    }
    let ext_total = u16::from_be_bytes([buf[pos], buf[pos + 1]]) as usize;
    pos += 2;
    let end = (pos + ext_total).min(buf.len());

    while pos + 4 <= end {
        let ext_type = u16::from_be_bytes([buf[pos], buf[pos + 1]]);
        let ext_len = u16::from_be_bytes([buf[pos + 2], buf[pos + 3]]) as usize;
        pos += 4;
        if ext_type == 0 {
            if pos + 5 <= end {
                let name_len = u16::from_be_bytes([buf[pos + 3], buf[pos + 4]]) as usize;
                let name_start = pos + 5;
                if name_start + name_len <= end {
                    return String::from_utf8(buf[name_start..name_start + name_len].to_vec()).ok();
                }
            }
        }
        pos += ext_len;
    }
    None
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let pc_local_ip = match local_ip() {
        Ok(IpAddr::V4(ipv4)) => ipv4,
        _ => Ipv4Addr::new(127, 0, 0, 1),
    };

    println!("============================================================");
    println!("   PS5 DNS + SNI DPI BYPASS BRIDGE (Roblox & Discord)");
    println!("============================================================");
    println!("Tespit Edilen Yerel PC IP Adresiniz: {}", pc_local_ip);
    println!();
    println!("PS5 Ağ Ayarlarından Yapılacaklar:");
    println!("  * DNS Ayarları: Manuel");
    println!("  * Birincil DNS (Primary DNS): {}", pc_local_ip);
    println!("  * İkincil DNS (Secondary DNS): 1.1.1.1 (veya 8.8.8.8)");
    println!("  * Proxy Sunucusu: Kullanma (Do Not Use)");
    println!("============================================================");
    println!();

    let dns_socket = match UdpSocket::bind("0.0.0.0:53").await {
        Ok(s) => Arc::new(s),
        Err(e) => {
            eprintln!("[HATA] Port 53 (DNS) acilamadi: {}", e);
            eprintln!("Lutfen programi 'Yonetici Olarak Calistir' (Run as Administrator)!");
            return Err(e.into());
        }
    };
    println!("[+] DNS Servisi Port 53 uzerinde calisiyor.");

    let https_listener = match TcpListener::bind("0.0.0.0:443").await {
        Ok(l) => l,
        Err(e) => {
            eprintln!("[HATA] Port 443 acilamadi: {}", e);
            eprintln!("Lutfen port 443'u kullanan servisleri kapatin veya Yonetici yetkisi verin.");
            return Err(e.into());
        }
    };
    println!("[+] HTTPS Bypass Port 443 calisiyor.");

    let http_listener = TcpListener::bind("0.0.0.0:80").await?;
    println!("[+] HTTP Bypass Port 80 calisiyor.");
    println!("[+] Servisler hazir, PS5 baglantilari bekleniyor...\n");

    let dns_sock_clone = Arc::clone(&dns_socket);
    tokio::spawn(async move {
        let mut buf = [0u8; 1024];
        loop {
            if let Ok((len, peer)) = dns_sock_clone.recv_from(&mut buf).await {
                let sock = Arc::clone(&dns_sock_clone);
                let packet = buf[..len].to_vec();
                tokio::spawn(async move {
                    handle_dns(sock, packet, peer, pc_local_ip).await;
                });
            }
        }
    });

    tokio::spawn(async move {
        loop {
            if let Ok((stream, peer)) = http_listener.accept().await {
                tokio::spawn(handle_plain_http(stream, peer));
            }
        }
    });

    loop {
        let (stream, peer) = https_listener.accept().await?;
        tokio::spawn(handle_tls_proxy(stream, peer));
    }
}

async fn handle_dns(socket: Arc<UdpSocket>, data: Vec<u8>, peer: SocketAddr, pc_ip: Ipv4Addr) {
    if let Ok(request) = Message::from_vec(&data) {
        let mut response = Message::new();
        response.set_id(request.id());
        response.set_message_type(MessageType::Response);
        response.set_op_code(OpCode::Query);
        response.set_recursion_desired(request.recursion_desired());
        response.set_recursion_available(true);
        response.set_response_code(ResponseCode::NoError);

        if let Some(query) = request.queries().first() {
            let name_str = query.name().to_utf8();
            response.add_query(query.clone());

            if is_blocked_domain(&name_str) && (query.query_type() == RecordType::A) {
                println!("[DNS-SPOOF] Hedef Yakalandi: {} -> {}", name_str, pc_ip);
                let record = Record::from_rdata(
                    query.name().clone(),
                    60,
                    RData::A(hickory_proto::rr::rdata::A(pc_ip)),
                );
                response.add_answer(record);

                if let Ok(bytes) = response.to_vec() {
                    let _ = socket.send_to(&bytes, peer).await;
                }
                return;
            }

            if let Ok(lookup) = UPSTREAM_RESOLVER.lookup_ip(&name_str).await {
                for ip in lookup.iter() {
                    match ip {
                        IpAddr::V4(ipv4) if query.query_type() == RecordType::A => {
                            let rec = Record::from_rdata(
                                query.name().clone(),
                                300,
                                RData::A(hickory_proto::rr::rdata::A(ipv4)),
                            );
                            response.add_answer(rec);
                        }
                        IpAddr::V6(ipv6) if query.query_type() == RecordType::AAAA => {
                            let rec = Record::from_rdata(
                                query.name().clone(),
                                300,
                                RData::AAAA(hickory_proto::rr::rdata::AAAA(ipv6)),
                            );
                            response.add_answer(rec);
                        }
                        _ => {}
                    }
                }
                if let Ok(bytes) = response.to_vec() {
                    let _ = socket.send_to(&bytes, peer).await;
                }
            }
        }
    }
}

async fn handle_tls_proxy(mut client_stream: TcpStream, peer: SocketAddr) {
    let mut buf = [0u8; 4096];
    let n = match client_stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let sni = extract_sni(&buf[..n]).unwrap_or_else(|| "roblox.com".to_string());
    println!("[HTTPS-BYPASS] TLS Baglantisi: {} (Istemci: {})", sni, peer);

    let target_ip = match UPSTREAM_RESOLVER.lookup_ip(&sni).await {
        Ok(lookup) => match lookup.iter().next() {
            Some(ip) => ip,
            None => return,
        },
        Err(e) => {
            eprintln!("[HATA] {} cozulemedi: {}", sni, e);
            return;
        }
    };

    let target_addr = SocketAddr::new(target_ip, 443);
    if let Ok(mut target_stream) = TcpStream::connect(target_addr).await {
        let _ = target_stream.set_nodelay(true);
        let _ = client_stream.set_nodelay(true);

        let split_at = if n > 10 { 3 } else { 1 };
        let part1 = &buf[..split_at];
        let part2 = &buf[split_at..n];

        let _ = target_stream.write_all(part1).await;
        let _ = target_stream.flush().await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _ = target_stream.write_all(part2).await;
        let _ = target_stream.flush().await;

        let _ = copy_bidirectional(&mut client_stream, &mut target_stream).await;
    }
}

async fn handle_plain_http(mut client_stream: TcpStream, _peer: SocketAddr) {
    let mut buf = [0u8; 4096];
    let n = match client_stream.read(&mut buf).await {
        Ok(n) if n > 0 => n,
        _ => return,
    };

    let request = String::from_utf8_lossy(&buf[..n]);
    let host = request
        .lines()
        .find(|l| l.to_lowercase().starts_with("host:"))
        .map(|l| l[5..].trim())
        .unwrap_or("roblox.com");

    if let Ok(lookup) = UPSTREAM_RESOLVER.lookup_ip(host).await {
        if let Some(ip) = lookup.iter().next() {
            if let Ok(mut target_stream) = TcpStream::connect(SocketAddr::new(ip, 80)).await {
                let _ = target_stream.write_all(&buf[..n]).await;
                let _ = copy_bidirectional(&mut client_stream, &mut target_stream).await;
            }
        }
    }
}
