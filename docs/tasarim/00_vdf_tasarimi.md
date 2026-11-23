# VDF — Vehicle Description Format: Tasarım Taslağı v0 (23.11.2026)

> Durum: **TASLAK, tartışmaya açık.** Teknik kimlik: crate/PyPI `nvdf`, dosya uzantısı `.nvdf` (içerik YAML).
> İlk kullanıcısı NSIM simülatörü (`D:\rust-gazebo`), ama format **simülatörden bağımsızdır**.

## 1. Neden yeni bir format?
| Format | Güçlü yanı | Eksiği |
|---|---|---|
| URDF | ROS ekosistemi | Sadece ağaç; sensör, dünya, kapalı zincir yok; makro için xacro gerekir |
| SDF | Dünya ve sensör tanımı | 12 şema sürümü, tipsiz eklenti XML'i, aero/itki eklentiye gömülü |
| MJCF | Robot kolu, aktüatör, tendon | Aero ve hidro yok |
| JSBSim XML | Uçak aerodinamiği, itki tabloları | Robot, sensör ve sahne yok |
| USD + UsdPhysics | Endüstri standardı, güçlü | Elle yazılmıyor, araç fiziği şeması yok |

**Boşluk:** Aero katsayı tabloları, pervane CT/CQ(J), ESC–motor–batarya zinciri, hidrodinamik katsayılar ve sensör gürültüsünü (Allan varyansı) **aynı yerde, tipli ve birimli** tanımlayan, elle yazılabilen ve araçlar arasında dönüştürülebilen bir format yok.

## 2. Hedefler / hedef olmayanlar
**Hedefler**
1. Sabit kanat, multikopter, VTOL, su altı ve su üstü araçları, kara robotu ve robot kolunu tek modelle tanımlamak.
2. Elle rahat yazılabilmek: YAML yazımı, açık birimler, kısa varsayılanlar.
3. Makineyle kesin doğrulanabilmek: JSON Schema ve anlamsal doğrulayıcı.
4. Dönüştürülebilmek: URDF, SDF, MJCF ve JSBSim'den içe ve dışa.
5. Her dilden kullanılabilmek: Rust referans kütüphanesi, Python paketi, CLI, JSON Schema.
6. Kararlı olmak: Baştan semver ve otomatik sürüm geçişi.

**Hedef olmayanlar (v1'e kadar)**
- Görsel sahne/render formatı olmak (geometri için glTF/OBJ/STL'ye referans verilir).
- Davranış ve kontrol mantığı tanımlamak (otopilot parametreleri ve görevler kapsam dışı).
- Turing-tam bir makro dili (xacro'nun sorunu). Yalnızca parametre ikamesi ve şablon desteklenir.

## 3. Temel kurallar
### 3.1 Belge yapısı
Her dosya tek bir belgedir ve en üstte sürüm ile tür bulunur:
```yaml
nvdf: 0.1
kind: model        # model | world | component
name: rascal110
```
- **model:** Bir araç veya robot (gövdeler, eklemler, alan modülleri).
- **world:** Ortam (arazi, atmosfer, su, yerçekimi, konum) ve içindeki modeller.
- **component:** Yeniden kullanılabilir parça: motor, pervane, batarya, kanat profili, sensör. **Ortak bir parça kataloğu** (örn. "APC 10x4.7", "T-Motor MN3110 700Kv") bu sayede mümkün olur.

### 3.2 Birimler
- **Sayılar her zaman SI'dır.** Açı için tek istisna yok: düz sayı radyan demektir.
- **Birimli metin kabul edilir ve yükleme anında SI'ya çevrilir:** `"4.2 kg"`, `"30 deg"`, `"1500 mAh"`, `"700 rpm/V"`.
- Bilinmeyen veya boyutu yanlış birim **hata verir** (örn. uzunluk alanına `"3 kg"`).

### 3.3 Çerçeveler (öneri, karar bekliyor)
- Saklama kuralı **FLU gövde / ENU dünya** (ROS REP-103) olsun. URDF, SDF ve MJCF dönüşümü kayıpsız olur.
- Havacılık modülleri kendi eksenlerini açıkça belirtir (`axes: frd`, `stability`). Dönüşüm kütüphanede tanımlı tek bir yerde yapılır.
- Poz: `pose: {xyz: [..], rpy: [..]}` veya `quat`. Varsayılan referans ebeveyn gövdedir; `relative_to: <çerçeve>` ile değiştirilebilir.
- **Tek çerçeve çizgesi**, yükleme anında çözülür; döngü varsa hata verir. (SDF'nin `relative_to` / `attached_to` ikiliği yok.)

### 3.4 Bileşim
```yaml
params:
  arm_length: {type: length, default: "0.25 m"}
include:
  - uri: catalog://motors/tmotor-mn3110-700kv
    as: motor
  - uri: ./rotor.nvdf
    as: rotor_fl
    with: {arm: ${arm_length}, spin: ccw}
```
- `${param}` ikamesi ve basit aritmetik (`${arm_length * 0.707}`) desteklenir; döngü ve koşul yok.
- URI şemaları: göreli yol, `catalog://` (parça kataloğu), `https://`, `package://` (ROS uyumu).

### 3.5 Tablolar
Tablolar aero, pervane, batarya ve itici için ortak tiptir:
```yaml
CL:
  inputs: [alpha, flap]                 # bağımsız değişkenler (isimli)
  breakpoints: {alpha: "-10 deg .. 20 deg step 2 deg", flap: [0, 0.35]}
  values: [[...], [...]]                # veya file: cl.csv
  interp: linear                        # linear | cubic | nearest
  extrapolate: clamp                    # clamp | linear | error
```

### 3.6 Genişletme
- `x-<üretici>-<alan>` anahtarları serbesttir. Doğrulayıcı bunları yok saymaz, olduğu gibi korur ve dönüştürmede taşır.
- Bilinmeyen standart anahtar **hata verir** (yazım hatası sessizce geçmesin).

## 4. Modüller
| Modül | İçerik (v0.1 → sonrası) |
|---|---|
| **core** | body (kütle, CoM, atalet), joint (fixed, revolute, prismatic, continuous, ball, floating; limit, sönüm, sürtünme), geometry (box, sphere, cylinder, capsule, mesh, heightfield), collision/visual, material |
| **sensors** | imu, gnss, baro, mag, airspeed, rangefinder, camera, lidar. Gürültü **yoğunluk cinsinden** (Allan: ARW, bias instability, RRW), gecikme, jitter, örnekleme hızı |
| **aero** | Lifting surface (span, chord, sweep, dihedral, airfoil) veya katsayı tabloları CL/CD/CY/Cl/Cm/Cn(α, β, δ, p̂, q̂, r̂); kontrol yüzeyleri ve eklemle bağlantı; referans S, b, c̄ |
| **propulsion** | propeller (D, pitch, CT/CQ(J) tablosu veya BEMT geometrisi), rotor (spin yönü, eksen), motor (Kv, R, I0, atalet), ESC (PWM aralığı, slew), itici (su altı K_T/K_Q) |
| **power** | battery (hücre sayısı, kapasite, OCV(SoC) tablosu, R0, RC çifti), güç ağı (hangi ESC hangi bataryaya bağlı) |
| **actuators** | servo (hız, tork, gecikme, ölü bölge), kanal eşleme (otopilot çıkışı → eklem veya motor) |
| **hydro** (v0.2) | eklenmiş kütle 6×6, doğrusal + karesel sönüm, hacim/CoB, itici |
| **ground** (v0.2) | tekerlek, lastik (fırça/Pacejka), süspansiyon, palet |
| **world** | atmosfer (ISA + rüzgar + türbülans), arazi, su yüzeyi, coğrafi orijin (WGS84), modellerin yerleşimi |

## 5. Örnek (kısaltılmış quad)
```yaml
nvdf: 0.1
kind: model
name: quad-x-450
params:
  arm: {type: length, default: "0.225 m"}
bodies:
  frame:
    mass: "1.45 kg"
    inertia: {ixx: 0.0123, iyy: 0.0123, izz: 0.0224}
    collision: {box: {size: [0.3, 0.3, 0.1]}}
    visual: {mesh: {uri: ./meshes/f450.glb}}
sensors:
  imu0: {kind: imu, body: frame, rate: "1000 Hz",
         gyro: {arw: "0.15 deg/sqrt(h)", bias_instability: "5 deg/h"},
         accel: {vrw: "0.05 m/s/sqrt(h)"}}
power:
  battery: {use: catalog://batteries/4s-5000mah-lipo}
propulsion:
  rotors:
    - {name: m1, body: frame, pose: {xyz: ["${arm*0.707}", "${-arm*0.707}", 0]}, spin: ccw,
       motor: catalog://motors/tmotor-mn3110-700kv, prop: catalog://props/apc-10x4.7,
       esc: {pwm: [1000, 2000]}, battery: battery}
    # m2..m4 …
actuators:
  channels: {1: m1, 2: m2, 3: m3, 4: m4}   # ArduPilot SERVOx → rotor
```

## 6. Araçlar
- **Kütüphane:** `nvdf` crate'i. Okuma, yazma, birim çözümü, include/param çözümü, doğrulama, çözülmüş "düz model" çıktısı. Simülatörler bu düz modeli kullanır.
- **CLI:** `nvdf validate | fmt | resolve | convert | schema | inspect`. `inspect` toplam kütle, CoM, atalet, itki/ağırlık oranı ve kanat yüklemesi gibi değerleri hesaplar.
- **Python:** `pip install nvdf` (PyO3 + maturin). Aynı Rust çekirdeği kullanılır.
- **JSON Schema:** Rust tiplerinden üretilir (`schemars`). VS Code YAML eklentisiyle otomatik tamamlama ve satır içi hata gösterimi sağlar.
- **Anlamsal doğrulama:**
  - kütle > 0
  - atalet pozitif tanımlı ve üçgen eşitsizliğini sağlıyor
  - çerçeve döngüsü yok
  - tablolar monoton ve boyutları tutarlı
  - kanal çakışması yok
  - birim boyutları doğru

## 7. Dönüştürücüler (öncelik sırası)
| Yön | Format | Sürüm | Not |
|---|---|---|---|
| içe | URDF | v0.1 | çekirdek modül |
| içe | JSBSim XML | v0.1 | aero + itki tabloları → Rascal buradan |
| dışa | URDF, SDF | v0.1 | aero/itki `x-` uzantısı olarak veya kayıp uyarısıyla |
| içe | AeroSandbox | v0.2 | Bertaraf'ın aero verisi |
| içe/dışa | SDF, MJCF | v0.2 | |
| dışa | USD (UsdPhysics) | v0.3+ | |
Kayıplı dönüşümlerde **her kayıp raporlanır**, sessizce atlanan bir şey olmaz.

## 8. Repo düzeni (öneri)
```
crates/nvdf/          model tipleri, ayrıştırma, birimler, çözümleme, doğrulama
crates/nvdf-convert/  içe/dışa dönüştürücüler (feature bayraklarıyla)
crates/nvdf-cli/      komut satırı aracı
python/               PyO3 bağlaması (maturin)
schema/               üretilmiş JSON Schema (sürüm başına)
spec/                 insan için spesifikasyon (markdown, normatif)
catalog/              örnek parça kataloğu (motor, pervane, batarya)
examples/             rascal110, quad-x-450, …
tests/conformance/    geçerli/geçersiz örnek seti — başka dillerdeki uygulamalar da kullanabilir
```
Lisans: MIT + Apache-2.0.

## 9. v0.1 kapsamı (NSIM MVP ile paralel)
- Modüller: core, sensors (imu, gnss, baro, mag, airspeed), aero (tablo tabanlı), propulsion (pervane tablosu, BLDC, ESC), power (batarya), actuators (kanal eşleme), world (atmosfer + düz zemin + coğrafi orijin).
- Araçlar: kütüphane, `validate`/`fmt`/`resolve`/`inspect`/`schema` CLI, JSON Schema.
- Dönüştürücüler: içe URDF, içe JSBSim, dışa URDF.
- Örnek modeller: **Rascal 110** (JSBSim'den dönüştürülmüş), **quad-x-450**.

## 10. Kolay kullanım: kademeli ayrıntı (öneri, 23.11.2026)
Kullanıcı ne kadar bilgi verirse o kadar gerçekçi model alır. Hiçbir şey vermediği yerde makul bir varsayılan kullanılır ve **bu açıkça raporlanır**.

| Seviye | Kullanıcı ne yazar | Örnek |
|---|---|---|
| **0. İçe aktarma** | Hiçbir şey; mevcut model | `nvdf convert iris.sdf`, JSBSim, URDF |
| **1. Şablon** | 5–10 parametre | `template: quad-x` + kütle, kol, parça adları |
| **2. Katalog** | Parça adı | `prop: catalog://props/apc-10x4.7` |
| **3. Parametrik** | Birkaç sayı | motor `kv: 700`, batarya `cells: 4, capacity: 5000 mAh` → eğriler varsayılan |
| **4. Tam** | Tablolar, Allan parametreleri | CT/CQ(J), OCV(SoC), R0 + RC, gürültü yoğunlukları |

**Kurallar:**
- **Sadakat seviyesi modelin parçasıdır.** Motor `simple` (maks. devir + gecikme, Gazebo benzeri) ya da `electric` (Kv, R, I0, batarya bağlantısı) olabilir; sensör `grade: consumer | industrial | tactical` ya da tam parametreler.
- **Her türetilmiş değerin kaynağı izlenir:** `user` / `template` / `catalog` / `estimated`. `nvdf inspect` bunları listeler; `estimated` değerler uyarıyla gösterilir. "Sessiz tahmin yok" ilkesinin kullanıcı dostu hali budur.
- **Şablonlar bir VDF belgesinin parametreli halidir** (`params` + `include`). Ayrı bir dil değildir; kullanıcı istediğinde şablonu açıp her ayrıntıyı ezebilir.
- **Tahmin yöntemleri belgelenir ve test edilir.** Örneğin geometriden kanat katsayıları: şerit teorisi / VLM; çap ve hatveden pervane eğrisi: genel ampirik form. Doğruluk sınırları belirtilir.
- `nvdf new quad-x` gibi bir iskelet komutu, sonra NSIM görüntüleyicisinde bir sihirbaz.

## 11. Kararlar (kullanıcı, 23.11.2026)
1. **Saklama çerçevesi: FLU gövde / ENU dünya** (ROS REP-103). Havacılık modülleri kendi eksenlerini (`frd`, `stability`) açıkça belirtir. NED/FRD dönüşümü tüketici tarafında yapılır (örn. NSIM'in ArduPilot köprüsü).
2. **Açı birimi: radyan.** Birimsiz her sayı SI'dır. Derece için `"30 deg"` yazılır.
3. **Parça kataloğu: ayrı repo `nvdf-catalog`.** `catalog://` adresleri buraya çözülür, katalog format sürümlerinden bağımsız güncellenir.
4. **İfade dili: `${param}` + dört işlem + parantez**, bir de küçük bir güvenli fonksiyon kümesi (`sqrt`, `sin`, `cos`, `tan`, `atan2`, `min`, `max`, `abs`, `pi`). Döngü ve koşul yok.

## 12. Açık konular
- Lisans: MIT + Apache-2.0 (öneri).
- `nvdf-catalog` için katkı kuralları: veri kaynağı, ölçüm koşulları, lisans (üretici verisi telifli olabilir).
