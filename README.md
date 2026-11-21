# 🎮 PS5 DNS & SNI DPI Bypass Bridge (Roblox & Discord)

[🇹🇷 Türkçe](#-türkçe-rehber) | [🇬🇧 English](#-english-guide)

---

## 🇹🇷 Türkçe Rehber

Bu proje, PlayStation 5 (PS5) ve diğer oyun konsollarında Türkiye gibi ülkelerde erişim engeli bulunan **Roblox** ve **Discord** gibi platformlara, konsola herhangi bir jailbreak veya harici yazılım yüklemeden erişebilmenizi sağlayan yerel bir köprüdür (DNS & SNI Proxy Bridge).

### 💡 Nasıl Çalışır ve Neden Gereklidir?
1. **Roblox Konsol Kısıtlaması:** PS5'teki Roblox uygulaması işletim sisteminin HTTP Proxy ayarlarını yok sayar. Bu yüzden standart HTTP/HTTPS proxy'ler çalışmaz.
2. **DNS & SNI Blokajı:** Sadece DNS değiştirmek de yetersizdir; çünkü internet servis sağlayıcılarının (İSS) DPI (Deep Packet Inspection) sistemleri, TLS ClientHello paketindeki SNI bilgisini (`roblox.com`, `discord.com`) okuduğu anda bağlantıyı keser.
3. **Çözümümüz:** 
   - Bilgisayarınızda yerel bir DNS sunucusu (Port 53) ve TLS SNI Proxy (Port 443) çalıştırır.
   - PS5 sadece bilgisayarınızı DNS olarak görür. PS5 Roblox veya Discord sorgusu yaptığında, DNS sunucusu hedef IP olarak bilgisayarınızın yerel IP'sini döner (DNS Spoofing).
   - PS5 doğrudan bilgisayarınıza bağlanır. Yazılımımız gelen TLS paketlerini **SNI Fragmentation (Parçalama)** yöntemiyle bölerek Türkiye'deki DPI sansürünü deler ve gerçek sunuculara iletir.
   - Diğer tüm normal trafik (PlayStation Network, oyun indirmeleri vs.) doğrudan Cloudflare (1.1.1.1) ve Google DNS (8.8.8.8) üzerinden kesintisiz akar.

> ⚠️ **GoodbyeDPI Gereksinimi Hakkında:**
> Yazılımımız kendi içinde temel TLS SNI Fragmentation yapabilmektedir. Ancak oyun içi canlı sunuculara (UDP bağlantıları) veya ISP'nizin ağır IP bazlı kısıtlamalarına tam çözüm sağlamak için arka planda **GoodbyeDPI**'ın da açık tutulması (veya bu yazılımla eşzamanlı kullanılması) maksimum stabilite ve bağlantı kalitesi sağlar.

---

### 🚀 Kurulum ve Kullanım Adımları

#### 1. Bilgisayarda Hazırlık
1. İndirdiğiniz veya derlediğiniz `ps5_dns_bridge.exe` dosyasına **sağ tıklayın** ve **"Yönetici Olarak Çalıştır"** (Run as Administrator) seçeneğine basın.
   *(53 numaralı DNS portunu ve 443 numaralı HTTPS portunu dinlemek için Windows yönetici izni şarttır).*
2. Program açıldığında terminal penceresinde kendi yerel IP adresinizi otomatik olarak tespit edip gösterecektir:
   ```text
   Tespit Edilen Yerel PC IP Adresiniz: 192.168.1.xxx
   ```
3. Windows Güvenlik Duvarı onay kutusu çıkarsa **"Erişime İzin Ver" (Allow)** butonuna tıklayın.

#### 2. PS5 Ağ Ayarları
1. PS5 menüsünden: **Ayarlar (Settings)** -> **Ağ (Network)** -> **Ayarlar (Settings)** -> **İnternet Bağlantısını Ayarla (Set Up Internet Connection)**.
2. Bağlı olduğunuz Wi-Fi veya Kablolu ağın üzerine gelin.
3. Kumandadaki **Options (3 Çizgi)** tuşuna basıp **Gelişmiş Ayarlar (Advanced Settings)** seçin.
4. Ayarları şu şekilde yapın:
   * **DNS Ayarları (DNS Settings):** `Manuel (Manual)`
   * **Birincil DNS (Primary DNS):** Terminalde yazan PC IP Adresiniz (örn: `192.168.1.xxx`)
   * **İkincil DNS (Secondary DNS):** `1.1.1.1` veya `8.8.8.8`
   * **Proxy Sunucusu (Proxy Server):** `Kullanma (Do Not Use)`
5. **Tamam (OK)** diyerek kaydedin.

🎉 **Tebrikler!** Artık PS5'inizden Roblox'u ve Discord sesli sohbetini kesintisiz kullanabilirsiniz.

---

### 🛠️ Kaynak Koddan Derleme (Geliştiriciler İçin)
Bilgisayarınızda Rust ve Cargo kuruluysa:
```bash
cargo build --release
```
Derlenen `.exe` dosyası `target/release/ps5_dns_bridge.exe` konumunda olacaktır.

---

## 🇬🇧 English Guide

A zero-configuration, lightweight DNS and SNI reverse proxy written in Rust that allows gaming consoles (PlayStation 5, Xbox, Nintendo Switch) to bypass regional DPI blocks (such as Roblox and Discord bans in Turkey).

### 💡 How It Works
- **DNS Spoofing for Blocked Targets:** When the console asks for blocked domains (`*.roblox.com`, `*.discord.com`), our local DNS engine resolves them to your local PC.
- **SNI Fragmentation (DPI Bypass):** When the console connects over TLS on port 443, the proxy extracts the ClientHello packet and splits it across multiple TCP packets with a slight delay. This fragments the SNI, blinding ISP DPI hardware.
- **Pass-through for Everything Else:** PlayStation Network, PS Store, and non-blocked games are directly resolved using upstream secure DNS servers (Cloudflare 1.1.1.1 & Google 8.8.8.8).

---

### 🚀 Instructions

#### Step 1: Run the Bridge on Your PC
1. Right click on `ps5_dns_bridge.exe` and select **"Run as Administrator"** (required for binding ports 53 and 443).
2. The terminal will display your automatically detected local IPv4 address (e.g. `192.168.1.103`).
3. Make sure to allow network permissions if Windows Firewall prompts.

#### Step 2: Configure Your PS5 Network Settings
1. On your PS5, go to **Settings** -> **Network** -> **Settings** -> **Set Up Internet Connection**.
2. Highlight your connected Wi-Fi or LAN network and press the **Options button (three lines)** -> **Advanced Settings**.
3. Set:
   * **DNS Settings:** `Manual`
   * **Primary DNS:** Your PC's Local IP shown in the console (e.g. `192.168.1.103`)
   * **Secondary DNS:** `1.1.1.1` (or `8.8.8.8`)
   * **Proxy Server:** `Do Not Use`
4. Save and connect.

---

### 📦 Build from Source
```bash
git clone <repo_url>
cd ps5_dns_bridge
cargo build --release
```
Target binary: `target/release/ps5_dns_bridge.exe`

---

### ⚖️ License
MIT License. Created for educational and network research purposes.

