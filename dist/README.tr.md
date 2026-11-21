SWISS KIT
=========

(English: see README.md.)

YouTube ve TikTok videolarını MP4 veya MP3 olarak indirin; görselleri, sesleri
ve videoları dönüştürün (WEBP'den JPG'ye, MOV'dan MP4'e, WAV'dan MP3'e ve
daha fazlası). Her şey kendi bilgisayarınızda çalışır. Hiçbir şey internete
yüklenmez.


GEREKENLER (tek seferlik kurulum)
---------------------------------

Swiss Kit, işin ağır kısmı için üç ücretsiz araç kullanır:

  - yt-dlp   videoları indirir
  - ffmpeg   MP3 yapar ve dosyaları dönüştürür
  - deno     yt-dlp'nin YouTube sayfalarını okumasını sağlar

Bunları almanın en kolay yolu:

  1. Bu klasördeki "Install tools.bat" dosyasına çift tıklayın.
  2. "All done!" yazana kadar bekleyin. Her şeyi kendisi kurar (winget veya
     Scoop ile) ve bir indirme kesilirse yeniden dener. Hata bildirirse
     dosyayı tekrar çalıştırmanız yeterli.

Yalnızca görsel dönüştürmek istiyorsanız (WEBP'den JPG'ye gibi) bu adımı
atlayabilirsiniz. Görsel dönüştürme için bu araçların hiçbiri gerekmez.

Kurulum dosyası çalışmazsa
--------------------------
Normal bir PowerShell penceresi açın ("Yönetici olarak çalıştır" değil) ve şu
satırları sırayla çalıştırın:

    Set-ExecutionPolicy -Scope CurrentUser RemoteSigned
    irm get.scoop.sh | iex
    scoop install yt-dlp ffmpeg deno

İndirme yarıda kesilirse (bağlantı sıfırlandı, "could not be resolved" gibi),
aynı komutu tekrar çalıştırın. Hâlâ olmuyorsa kurulum için başka bir ağ ya da
VPN deneyin. Kurulum bitince VPN'i kapatabilirsiniz.


NASIL BAŞLATILIR
----------------

  1. Zip dosyasının tamamını çıkarın (sağ tıklayın, "Tümünü ayıkla").
     Programı zip'in içinden çalıştırmayın.
  2. "Start Swiss Kit.bat" dosyasına çift tıklayın.
  3. Tarayıcınız Swiss Kit ile açılır. Uygulamayı kullandığınız sürece siyah
     pencereyi açık tutun. O pencereyi kapatmak uygulamayı kapatır.

İlk seferde Windows "Windows bilgisayarınızı korudu" diyebilir. Bunun nedeni
uygulamanın yeni olması ve büyük bir şirket tarafından imzalanmamış olmasıdır.
"Daha fazla bilgi"ye, ardından "Yine de çalıştır"a tıklayın. Bir arkadaşınızdan
gelen dosyaya güvenmek istemezseniz, uygulamayı kaynak koddan kendiniz
derleyebilirsiniz (bu dosyanın sonuna bakın).

Swiss Kit yalnızca kendi bilgisayarınızı (127.0.0.1) dinler. Diğer cihazlar ve
web siteleri onu kullanamaz.


KULLANIMI
---------

Video indirmek
  1. Solda bir site seçin (aşağıdaki "Desteklenen siteler" bölümüne bakın).
  2. Bağlantıyı yapıştırın ve "Getir"e tıklayın.
  3. MP4 (video) veya MP3 (yalnızca ses) seçin. MP4 için kaliteyi de
     seçebilirsiniz.
  4. "MP4 indir" (ya da "MP3 indir") düğmesine tıklayın. İlerlemeyi sıradan
     izleyin.
  5. "Bitti" yazınca "Klasörde göster"e tıklayın.

Desteklenen siteler
  YouTube, TikTok, Instagram, X (Twitter), Facebook, Vimeo, Reddit,
  SoundCloud, Twitch, Dailymotion ve Pinterest'in kendi düğmeleri var.
  "Herhangi bir site", yt-dlp'nin tanıdığı hemen hemen her video veya ses
  sitesinden bir bağlantıyı kabul eder. Site desteklenmiyorsa bir hata
  görürsünüz.

  Bilmeniz gerekenler:
  - Yalnızca herkese açık gönderiler çalışır. Özel gönderiler ya da giriş
    yapmanızı isteyenler (Instagram, Facebook ve X'te sık görülür)
    indirilemez.
  - SoundCloud yalnızca ses içerir, bu yüzden her zaman MP3 kaydeder.
  - Oynatma listeleri ve tüm kanallar desteklenmez. Bağlantıları tek tek
    yapıştırın.
  - Siteler sık sık değişir. Biri çalışmayı bırakırsa yt-dlp'yi güncelleyin
    (aşağıya bakın).

Dil
  Sol çubuğun altındaki EN / TR anahtarını kullanın. Swiss Kit seçiminizi
  hatırlar. İlk açılışta tarayıcınız Türkçeyse Türkçe seçilir.

Dosya dönüştürmek
  1. Solda Görseller, Ses veya Video'yu seçin.
  2. Neye dönüştüreceğinizi seçin (örneğin JPG).
  3. Dosyalarınızı kutunun üzerine sürükleyin ya da kutuya tıklayıp dosya
     seçin. Birden çok dosyayı aynı anda seçebilirsiniz.

"Hızlı dönüştür" (sağ altta) sizi sık yapılan işlere, örneğin WEBP'den JPG'ye ya
da MP4'ten MP3'e, doğrudan götürür.

Dosyalarım nereye gidiyor?
  Downloads\Swiss Kit  (İndirilenler klasörünüzün içinde).
  Kenar çubuğu tam klasörü her zaman gösterir ve "Klasörü aç" onu açar.
  Swiss Kit hiçbir dosyanın üzerine yazmaz. "photo.jpg" zaten varsa yenisi
  "photo (1).jpg" olarak kaydedilir.

Neleri dönüştürebilir?
  Görseller  WEBP, PNG, JPG, GIF, BMP, TIFF (birbirlerine)
  Ses        MP3, WAV, M4A, OGG, FLAC
  Video      MP4, WEBM, GIF veya sesi MP3 olarak ayıklama
  Henüz yok  iPhone'daki HEIC fotoğraflar


BİR SORUN ÇIKARSA
-----------------

yt-dlp ve ffmpeg bulunduğunda kenar çubuğunda yanlarında yeşil nokta görünür.

Kırmızı nokta veya "Not found on this PC"
  "Install tools.bat" dosyasını çalıştırın. Araçları az önce kurduysanız
  bilgisayarı bir kez yeniden başlatın ki Windows onları tanısın, sonra Swiss
  Kit'i tekrar açın.

İndirme başarısız oluyor ya da YouTube birden çalışmayı bırakıyor
  YouTube sık sık değişir ve yt-dlp'nin buna yetişmesi gerekir. Güncelleyin,
  sonra tekrar deneyin:
      winget upgrade yt-dlp.yt-dlp
  ya da scoop kullandıysanız:
      scoop update yt-dlp

"Bu bir ... bağlantısı değil"
  Yanlış site düğmesindesiniz. Solda doğru siteyi seçin ya da "Herhangi bir
  site"yi kullanın. Oynatma listeleri desteklenmez. Tek bir videonun
  bağlantısını yapıştırın.

Instagram, Facebook veya X'te indirme başarısız oluyor
  Bu siteler, giriş yapmamış kişilerden gönderileri sık sık gizler. Herkese
  açık gönderiler genellikle çalışır. Özel olanlar hiçbir zaman çalışmaz.

Tarayıcı açılmadı
  Siyah pencereye bakın. Adresi yazar, genellikle http://127.0.0.1:7878 olur.
  Onu tarayıcınıza yazın.

Siyah pencere hemen kapanıyor
  Önce zip'i çıkardığınızdan ve "swiss-kit-app.exe" dosyasının "Start Swiss
  Kit.bat" ile aynı klasörde durduğundan emin olun.

Windows Defender veya bir antivirüs dosyayı engelliyor
  Bu, yeni ve imzasız herhangi bir programda olabilir. Nereden geldiğine
  güveniyorsanız izin verin ya da kaynak koddan kendiniz derleyin.


ADİL KULLANIM
-------------

Yalnızca size ait olan, serbestçe kullanılabilen ya da kaydetmek için izniniz
bulunan içerikleri indirin. Her web sitesinin kurallarına ve videoyu yapan
kişilerin haklarına saygı gösterin.


KAYNAK KODDAN DERLEMEK (merak edenler için)
-------------------------------------------

Rust gerekir (https://rustup.rs). Windows'ta Rust ayrıca C++ derleme araçlarına
ihtiyaç duyar; MinGW ile GNU toolchain de çalışır (scoop install mingw).

  1. Kaynak klasörünü bir terminalde açın.
  2. "build-release.bat" dosyasına çift tıklayın ya da şunu çalıştırın:
         cargo build --release -p swiss-kit-app
  3. Program target\release\swiss-kit-app.exe dosyasıdır.

Komut satırı sürümü (tarayıcısız). "get" siteyi kendisi bulur:

    cargo run -p swiss-kit -- get "BAGLANTI" -f mp3 -o "%USERPROFILE%\Downloads"
    cargo run -p swiss-kit -- image photo.webp jpg
    cargo run -p swiss-kit -- media clip.mov mp4

Uygulama seçenekleri:  swiss-kit-app.exe --out "D:\Klasorum"  (kayıt klasörünü
seçer)  ve  --no-open  (tarayıcıyı otomatik açmaz).

Mac ve Linux: uygulama aynı cargo komutuyla derlenir ve aynı araçları kullanır
(brew install yt-dlp ffmpeg deno). Mac'te denenmemiştir.
