# MTProxy Desk ⚡ – مدير بروكسيات تليجرام لسطح المكتب
### Telegram Proxy Manager (v1.0.2)

[![Version](https://img.shields.io/badge/version-1.0.2-blue.svg)](https://github.com/moshraheem-sudo/MTProxy-Desk/releases)
[![Platform](https://img.shields.io/badge/platform-Windows%2010%20%7C%2011%20(64--bit)-0078d7.svg)](https://github.com/moshraheem-sudo/MTProxy-Desk)
[![Rust](https://img.shields.io/badge/engine-Rust%20%2B%20Tokio-DEA584.svg)](https://www.rust-lang.org/)
[![Developer](https://img.shields.io/badge/Developer-Mohammed%20Shaheed%20Al--Aajeebi-10b981.svg)](https://github.com/moshraheem-sudo)

---

## 🇸🇦 الدليل التعريفي الشامل (باللغة العربية)

تطبيق **MTProxy Desk** هو برنامج متطور واحترافي لبيئة Windows (64 بت) لإدارة وفحص وربط بروكسيات تليجرام **MTProto** الحية وفائقة السرعة بنقرة واحدة، تم تصميمه وبرمجته بواسطة **محمد شهيد الاعاجيبي (Mohammed Shaheed Al-Aajeebi)**.

### 🌟 أبرز مميزات الإصدار v1.0.2:

1. **جلب متزامن مزدوج للمصادر (Dual-Source Sync):**
   - **المصدر الأساسي 1:** مستودع [SoliSpirit/mtproto](https://github.com/SoliSpirit/mtproto).
   - **المصدر الثانوي 2:** مستودع [mtproto-auto-proxies](https://github.com/moshraheem-sudo/mtproto-auto-proxies) التلقائي السحابي المحدث كل ساعتين.
   - **تخطي الحظر في العراق والشرق الأوسط:** يتم سحب بروكسيات قناة تليجرام `https://t.me/s/ProxyMTProto` عبر سيرفرات GitHub السحابية، مما يضمن وصول أحدث البروكسيات للمستخدمين في العراق بدون أي حظر لمواقع تليجرام من مزودي الإنترنت المحليين.
   - **جلب واقعي ومؤكد لأحدث 50+ سيرفر:** يتم استخراج ما يزيد عن 50 سيرفر واقعي باستمرار دون إهمال أي مفتاح.

2. **منع التكرار الدقيق (Strict Deduplication):**
   - دمج ومطابقة السيرفرات بناءً على ثلاثية `(السيرفر، المنفذ، المفتاح السري)` وتوحيد روابط `tg://proxy` و `https://t.me/proxy` لمنع أي تكرار في الجدول.

3. **حذف السيرفرات المعطلة والمنتهية بضغطة زر (`🧹 Clean Dead`):**
   - إمكانية تنظيف قاعدة البيانات المحلية SQLite من أي بروكسيات غير متصلة أو متوقفة عن العمل فوراً مع الحفاظ التام على السيرفرات المفضلة.

4. **معرفة مصدر كل منفذ وسيرفر (Origin Feed):**
   - داخل نافذة تفاصيل السيرفر والمنفذ (`Port Info`)، يظهر المصدر الذي جُلب منه السيرفر بوضوح (سواء من الساحب التلقائي لقناة تليجرام أو من المستودع العالمي).

5. **نظام التحديث التلقائي البرمجي المدمج (In-App Updater):**
   - فحص مباشر مع GitHub Releases للإصدارات الأحدث مع إظهار حجم التحديث الحقيقي بالميغابايت ونسبة التحميل.
   - زر تثبيت وترقية فوري ينطلق بعد اكتمال التنزيل ويطلب أذونات المسؤول (UAC) لتثبيت النسخة الأحدث وحذف الملفات المؤقتة القديمة.

6. **واجهة خفيفة وفائقة السرعة:**
   - تصميم مضغوط ومناسب لكافة الشاشات وأجهزة الحواسيب واللابتوبات الضعيفة والقوية دون أي استهلاك لموارد الذاكرة أو المعالج.
   - ترتيب الأزرار بمرونة تامة (زر الربط بالتليجرام وزر الباركود قبل عمود اسم السيرفر لتسهيل الوصول).
   - دعم كامل للغتين: الإنجليزية كلغة افتراضية مع زر فوري للتحويل للغة العربية.
   - قائمة خيارات بالزر الأيمن للفأرة (Right-Click Context Menu) على أي سيرفر لتسهيل الربط السريع، إعادة الفحص، أو النسخ.

---

## 📦 حزم التحميل والتثبيت المباشرة (Direct Downloads)

| الملف | النوع | الوصف |
| :--- | :--- | :--- |
| **[MTProxy_Desk_Setup_v1.0.2.exe](file:///c:/Users/gahzi/Desktop/mtproto/MTProxy_Desk_Setup_v1.0.2.exe)** | برنامج تثبيت رسمي (NSIS) | يقوم بتثبيت البرنامج تلقائياً وإضافته لقائمة ابدأ وسطح المكتب مع كافة مكتبات التشغيل. |
| **[MTProxy_Desk_Setup_v1.0.2.msi](file:///c:/Users/gahzi/Desktop/mtproto/MTProxy_Desk_Setup_v1.0.2.msi)** | حزمة ويندوز القياسية (MSI) | مثبت Windows Installer قياسي مناسب للشركات والنشر العام. |
| **[MTProxy_Desk_Portable_v1.0.2.zip](file:///c:/Users/gahzi/Desktop/mtproto/MTProxy_Desk_Portable_v1.0.2.zip)** | نسخة محمولة جاهزة (Portable) | تعمل مباشرة بدون تثبيت، وتتضمن ملفات `mtproxy-desk.exe` و `WebView2Loader.dll` المدمج. |
| **[run_desktop.bat](file:///c:/Users/gahzi/Desktop/mtproto/run_desktop.bat)** | ملف تشغيل سريع فوري | يفحص البيئة والمكتبات تلقائياً ويشغل البرنامج بسلاسة. |

---

## 🇬🇧 English Documentation

**MTProxy Desk** is a high-performance, production-grade Windows desktop application built with **Rust**, **Tokio**, and **Tauri 2** for discovering, testing, benchmarking, and directly launching verified Telegram MTProto proxies.

### Key Capabilities:
- **Dual-Source High Availability:** Queries both `SoliSpirit/mtproto` and `moshraheem-sudo/mtproto-auto-proxies` simultaneously.
- **Bypasses Censorship via Cloud Runners:** GitHub Actions bridges Telegram web endpoints (`https://t.me/s/ProxyMTProto`) into raw feeds accessible without blocking in Iraq and restricted regions.
- **Strict Canonical Deduplication:** Guarantees zero duplicate entries across multiple feed formats.
- **1-Click Connect & Share Studio:** Direct link generation (`tg://proxy?...`) with QR Code generator for mobile phones.
- **Local SQLite Persistence:** Works offline with sub-second launch times and low RAM footprint (<40MB).
- **In-App GitHub Release Updater:** Automatically detects newer versions on GitHub, downloads installers with live MB progress, and self-installs cleanly.

---

## 👨‍💻 المطور / Developer & Credits

- **المطور / Developer:** محمد شهيد الاعاجيبي (Mohammed Shaheed Al-Aajeebi)
- **الإصدار الحالي / Version:** 1.0.2
- **مستودع المشروع / GitHub Repo:** [moshraheem-sudo/MTProxy-Desk](https://github.com/moshraheem-sudo/MTProxy-Desk)
- **مستودع البروكسيات التلقائي / Auto Scraper Feed:** [moshraheem-sudo/mtproto-auto-proxies](https://github.com/moshraheem-sudo/mtproto-auto-proxies)
- **قناة التليجرام المصدر / Telegram Source Channel:** [ProxyMTProto](https://t.me/s/ProxyMTProto)
