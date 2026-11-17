#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
أداة التحديث الفوري المباشر لبروكسيات Telegram من GitHub
تقوم بجلب أحدث قائمة من المستودع مباشرة وفحص استجابة الخوادم.
"""

import sys
import socket
import time
import urllib.request
import urllib.parse
from concurrent.futures import ThreadPoolExecutor

SOURCES = [
    ("SoliSpirit MTProto", "https://raw.githubusercontent.com/SoliSpirit/mtproto/master/all_proxies.txt"),
    ("mtproto-auto-proxies", "https://raw.githubusercontent.com/moshraheem-sudo/mtproto-auto-proxies/main/proxies.txt")
]

def test_proxy(server, port, timeout=3.0):
    start = time.time()
    try:
        s = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        s.settimeout(timeout)
        s.connect((server, port))
        ms = int((time.time() - start) * 1000)
        s.close()
        return True, ms
    except Exception:
        return False, None

def main():
    print("=" * 60)
    print(" MTProxy Desk - تحديث فوري مباشر من مصادر GitHub المزدوجة")
    for name, url in SOURCES:
        print(f" - {name}: {url}")
    print("=" * 60)
    print("\n[1/3] جاري تنزيل أحدث قوائم البروكسيات مع دعم Failover...")

    all_lines = []
    for name, url in SOURCES:
        try:
            req = urllib.request.Request(url, headers={'User-Agent': 'Mozilla/5.0'})
            with urllib.request.urlopen(req, timeout=10) as resp:
                text = resp.read().decode('utf-8', errors='ignore')
            lines = [l.strip() for l in text.splitlines() if l.strip() and not l.startswith('#')]
            print(f"✅ تم استلام {len(lines)} بروكسي من {name}.")
            all_lines.extend(lines)
        except Exception as e:
            print(f"⚠️ تعذر جلب {name}: {e} (الاعتماد على المصدر الآخر)")

    proxies = []
    seen = set()
    for line in all_lines:
        try:
            parsed = urllib.parse.urlparse(line if not line.startswith('t.me/') else 'https://' + line)
            qs = urllib.parse.parse_qs(parsed.query)
            srv = qs['server'][0].strip().rstrip('.').lower()
            port = int(qs['port'][0].strip())
            sec = qs['secret'][0].strip()
            key = f"{srv}:{port}:{sec}"
            if key not in seen:
                seen.add(key)
                proxies.append({'server': srv, 'port': port, 'secret': sec})
        except Exception:
            continue

    print(f"📊 إجمالي البروكسيات الموحدة بعد إزالة التكرار: {len(proxies)}")

    print(f"[2/3] تم تنظيف العناوين وإزالة التكرارات: {len(proxies)} بروكسي فريد.")
    print("\n[3/3] فحص استجابة الخوادم السريعة (TCP Ping)...")

    reachable = []
    def check(p):
        ok, ms = test_proxy(p['server'], p['port'])
        if ok:
            return {**p, 'ping': ms}
        return None

    with ThreadPoolExecutor(max_workers=30) as executor:
        for res in executor.map(check, proxies[:50]): # check first 50 immediately
            if res:
                reachable.append(res)
                print(f"  🟢 {res['server']}:{res['port']} - {res['ping']}ms")

    reachable.sort(key=lambda x: x['ping'])
    print("\n" + "=" * 60)
    print(f"🎯 أفضل وأسرع بروكسي متوفر حالياً:")
    if reachable:
        best = reachable[0]
        print(f"   الخادم: {best['server']}")
        print(f"   المنفذ: {best['port']}")
        print(f"   المفتاح: {best['secret']}")
        print(f"   زمن الاستجابة: {best['ping']} ms")
        print(f"\n   رابط الاتصال المباشر:")
        print(f"   tg://proxy?server={best['server']}&port={best['port']}&secret={best['secret']}")
    else:
        print("   لم يتم العثور على خادم متاح في العينة الأولى.")
    print("=" * 60)

if __name__ == '__main__':
    main()
