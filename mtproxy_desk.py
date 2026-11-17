#!/usr/bin/env python3
# -*- coding: utf-8 -*-
"""
MTProxy Desk – Telegram Proxy Manager (Python Edition)
الإصدار: 1.0.2
تطبيق احترافي متكامل لإدارة وفحص وتشغيل بروكسيات Telegram باستخدام Python و Tkinter
مع دعم التحديث الفوري المباشر من GitHub وقاعدة بيانات SQLite مدمجة.
"""

import sys
import os
import re
import time
import socket
import sqlite3
import threading
import urllib.request
import urllib.parse
import webbrowser
from datetime import datetime
from concurrent.futures import ThreadPoolExecutor

# GUI imports (Python standard library)
import tkinter as tk
from tkinter import ttk, messagebox, filedialog

PRIMARY_SOURCE_URL = "https://raw.githubusercontent.com/SoliSpirit/mtproto/master/all_proxies.txt"
SECONDARY_SOURCE_URL = "https://raw.githubusercontent.com/moshraheem-sudo/mtproto-auto-proxies/main/proxies.txt"
APP_DIR = os.path.dirname(os.path.abspath(__file__))
DB_PATH = os.path.join(APP_DIR, "mtproxy_desk.db")
LOG_PATH = os.path.join(APP_DIR, "MTProxyDesk_Diagnostics.log")

# --- Diagnostics Logger ---
def mask_secret(secret: str) -> str:
    if len(secret) <= 8:
        return "***MASKED***"
    return f"{secret[:4]}...{secret[-4:]} [MASKED]"

def log_event(level: str, category: str, message: str):
    timestamp = datetime.now().strftime("%Y-%m-%d %H:%M:%S.%f")[:-3]
    line = f"[{timestamp}] [{level}] [{category}] {message}"
    print(line)
    try:
        with open(LOG_PATH, "a", encoding="utf-8") as f:
            f.write(line + "\n")
    except Exception:
        pass

# --- Database Layer ---
class Database:
    def __init__(self, db_path=DB_PATH):
        self.db_path = db_path
        self._init_db()

    def _get_conn(self):
        return sqlite3.connect(self.db_path)

    def _init_db(self):
        with self._get_conn() as conn:
            c = conn.cursor()
            c.execute('''
                CREATE TABLE IF NOT EXISTS proxies (
                    id TEXT PRIMARY KEY,
                    server TEXT NOT NULL,
                    port INTEGER NOT NULL,
                    secret TEXT NOT NULL,
                    status TEXT NOT NULL,
                    ping_ms INTEGER,
                    last_check TEXT,
                    is_favorite INTEGER NOT NULL DEFAULT 0,
                    source_url TEXT NOT NULL
                )
            ''')
            c.execute('''
                CREATE TABLE IF NOT EXISTS settings (
                    key TEXT PRIMARY KEY,
                    value TEXT NOT NULL
                )
            ''')
            conn.commit()

    def upsert_proxies(self, proxies):
        with self._get_conn() as conn:
            c = conn.cursor()
            for p in proxies:
                c.execute('''
                    INSERT INTO proxies (id, server, port, secret, status, ping_ms, last_check, is_favorite, source_url)
                    VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)
                    ON CONFLICT(id) DO UPDATE SET
                        server = excluded.server,
                        port = excluded.port,
                        secret = excluded.secret,
                        source_url = excluded.source_url
                ''', (
                    p['id'], p['server'], p['port'], p['secret'],
                    p.get('status', 'Unchecked'), p.get('ping_ms'),
                    p.get('last_check'), 1 if p.get('is_favorite') else 0,
                    p.get('source_url', PRIMARY_SOURCE_URL)
                ))
            conn.commit()

    def get_all_proxies(self):
        with self._get_conn() as conn:
            c = conn.cursor()
            c.execute('SELECT id, server, port, secret, status, ping_ms, last_check, is_favorite, source_url FROM proxies')
            rows = c.fetchall()
            return [
                {
                    'id': r[0], 'server': r[1], 'port': r[2], 'secret': r[3],
                    'status': r[4], 'ping_ms': r[5], 'last_check': r[6],
                    'is_favorite': bool(r[7]), 'source_url': r[8]
                }
                for r in rows
            ]

    def update_proxy_status(self, proxy_id, status, ping_ms, last_check):
        with self._get_conn() as conn:
            c = conn.cursor()
            c.execute('UPDATE proxies SET status = ?, ping_ms = ?, last_check = ? WHERE id = ?',
                      (status, ping_ms, last_check, proxy_id))
            conn.commit()

    def toggle_favorite(self, proxy_id, is_fav):
        with self._get_conn() as conn:
            c = conn.cursor()
            c.execute('UPDATE proxies SET is_favorite = ? WHERE id = ?', (1 if is_fav else 0, proxy_id))
            conn.commit()

    def delete_proxy(self, proxy_id):
        with self._get_conn() as conn:
            c = conn.cursor()
            c.execute('DELETE FROM proxies WHERE id = ?', (proxy_id,))
            conn.commit()

    def get_setting(self, key, default=None):
        with self._get_conn() as conn:
            c = conn.cursor()
            c.execute('SELECT value FROM settings WHERE key = ?', (key,))
            row = c.fetchone()
            return row[0] if row else default

    def set_setting(self, key, value):
        with self._get_conn() as conn:
            c = conn.cursor()
            c.execute('INSERT INTO settings (key, value) VALUES (?, ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value',
                      (key, str(value)))
            conn.commit()

# --- Parser ---
def clean_server_host(raw_server: str) -> str:
    s = raw_server.strip()
    if s.startswith("http://"): s = s[7:]
    elif s.startswith("https://"): s = s[8:]
    if ":" in s and not s.startswith("["):
        s = s.split(":", 1)[0]
    s = s.strip("[]").rstrip(".")
    return s

def parse_proxy_line(line: str, source_url: str = PRIMARY_SOURCE_URL):
    line = line.strip()
    if not line or line.startswith("#"):
        return None
    if line.startswith("t.me/"):
        line = "https://" + line

    try:
        parsed = urllib.parse.urlparse(line)
        params = urllib.parse.parse_qs(parsed.query)
        server_list = params.get("server", [])
        port_list = params.get("port", [])
        secret_list = params.get("secret", [])

        if not (server_list and port_list and secret_list):
            return None

        server = clean_server_host(server_list[0])
        port = int(port_list[0].strip())
        secret = secret_list[0].strip()

        if not (1 <= port <= 65535) or not server or not secret:
            return None

        canonical_id = f"{server.lower()}:{port}:{secret}"
        return {
            'id': canonical_id,
            'server': server,
            'port': port,
            'secret': secret,
            'status': 'Unchecked',
            'ping_ms': None,
            'last_check': None,
            'is_favorite': False,
            'source_url': source_url
        }
    except Exception:
        return None

def parse_proxies_text(content: str, source_url: str = PRIMARY_SOURCE_URL):
    items = []
    seen = set()
    for line in content.splitlines():
        p = parse_proxy_line(line, source_url)
        if p and p['id'] not in seen:
            seen.add(p['id'])
            items.push(p) if hasattr(items, 'push') else items.append(p)
    return items

# --- Network Validator ---
def validate_proxy(proxy, timeout=4.0):
    host = proxy['server']
    port = proxy['port']
    start_time = time.time()

    # Step 1: DNS
    try:
        addrs = socket.getaddrinfo(host, port, socket.AF_UNSPEC, socket.SOCK_STREAM)
        if not addrs:
            return proxy['id'], "DnsFailed", None
        target_addr = addrs[0][4]
    except socket.gaierror:
        return proxy['id'], "DnsFailed", None
    except Exception:
        return proxy['id'], "Unreachable", None

    # Step 2: TCP Handshake
    s = socket.socket(socket.AF_INET if len(target_addr) == 2 else socket.AF_INET6, socket.SOCK_STREAM)
    s.settimeout(timeout)
    try:
        connect_start = time.time()
        s.connect(target_addr)
        elapsed_ms = int((time.time() - connect_start) * 1000)
        s.close()
        return proxy['id'], "TcpReachable", elapsed_ms
    except socket.timeout:
        s.close()
        return proxy['id'], "Timeout", None
    except Exception:
        s.close()
        return proxy['id'], "Unreachable", None

# --- Telegram Launcher ---
def launch_telegram(server: str, port: int, secret: str) -> dict:
    tg_url = f"tg://proxy?server={urllib.parse.quote(server)}&port={port}&secret={secret}"
    web_url = f"https://t.me/proxy?server={urllib.parse.quote(server)}&port={port}&secret={secret}"
    log_event("INFO", "LAUNCHER", f"Launching Telegram for {server}:{port} (Secret: {mask_secret(secret)})")

    opened = False
    try:
        if sys.platform == "win32":
            os.startfile(tg_url)
            opened = True
        else:
            opened = webbrowser.open(tg_url)
    except Exception as e:
        log_event("WARN", "LAUNCHER", f"Failed to launch tg:// url: {e}. Opening web fallback.")
        webbrowser.open(web_url)

    return {
        'tg_url': tg_url,
        'web_url': web_url,
        'success': opened
    }

# --- GUI Application ---
class MTProxyDeskApp:
    def __init__(self, root):
        self.root = root
        self.root.title("MTProxy Desk v1.0.2 – Dev: Mohammed Shaheed Al-Aajeebi")
        self.root.geometry("840x580")
        self.root.minsize(700, 460)
        self.root.configure(bg="#0b1120")

        self.db = Database()
        self.proxies = []
        self.is_validating = False
        self.search_query = ""
        self.filter_mode = "all"
        self.sort_mode = "ping"
        self.current_lang = "en"  # English default

        self._setup_styles()
        self._build_ui()
        self._load_local_data()

        # Start background 30-minute auto-update loop
        threading.Thread(target=self._auto_update_loop, daemon=True).start()

    def _setup_styles(self):
        style = ttk.Style()
        style.theme_use("clam")

        style.configure("Treeview",
                        background="#1e293b",
                        foreground="#f8fafc",
                        fieldbackground="#1e293b",
                        font=("Segoe UI", 10),
                        rowheight=32)
        style.map("Treeview",
                  background=[('selected', '#0284c7')],
                  foreground=[('selected', '#ffffff')])

        style.configure("Treeview.Heading",
                        background="#0f172a",
                        foreground="#94a3b8",
                        font=("Segoe UI", 10, "bold"),
                        relief="flat")
        style.map("Treeview.Heading",
                  background=[('active', '#1e293b')])

    def _build_ui(self):
        # Header
        header = tk.Frame(self.root, bg="#0f172a", height=60, padx=20, pady=10)
        header.pack(fill="x", side="top")

        brand_frame = tk.Frame(header, bg="#0f172a")
        brand_frame.pack(side="right" if self.current_lang == "ar" else "left")

        logo = tk.Label(brand_frame, text="⚡", font=("Segoe UI", 18, "bold"), fg="#38bdf8", bg="#0284c7", width=2, height=1)
        logo.pack(side="right" if self.current_lang == "ar" else "left", padx=8)

        titles = tk.Frame(brand_frame, bg="#0f172a")
        titles.pack(side="right" if self.current_lang == "ar" else "left")

        title_lbl = tk.Label(titles, text="MTProxy Desk – Telegram Proxy Manager", font=("Segoe UI", 14, "bold"), fg="#ffffff", bg="#0f172a")
        title_lbl.pack(anchor="e" if self.current_lang == "ar" else "w")

        sub_lbl = tk.Label(titles, text="إدارة وتحديث فوري لبروكسيات MTProto عبر محرك غير متزامن", font=("Segoe UI", 9), fg="#94a3b8", bg="#0f172a")
        sub_lbl.pack(anchor="e" if self.current_lang == "ar" else "w")

        # Top Buttons
        top_btn_frame = tk.Frame(header, bg="#0f172a")
        top_btn_frame.pack(side="left" if self.current_lang == "ar" else "right")

        self.btn_lang = tk.Button(top_btn_frame, text="English / عربي", font=("Segoe UI", 9, "bold"), bg="#1e293b", fg="#f8fafc",
                                  bd=0, padx=12, pady=6, cursor="hand2", command=self._toggle_language)
        self.btn_lang.pack(side="left", padx=4)

        btn_diag = tk.Button(top_btn_frame, text="🩺 السجل والتشخيص", font=("Segoe UI", 9, "bold"), bg="#1e293b", fg="#f8fafc",
                             bd=0, padx=12, pady=6, cursor="hand2", command=self._show_diagnostics)
        btn_diag.pack(side="left", padx=4)

        # Stats Cards Frame
        stats_frame = tk.Frame(self.root, bg="#0b1120", padx=20, pady=12)
        stats_frame.pack(fill="x")

        self.stat_cards = {}
        card_defs = [
            ("total", "إجمالي البروكسيات", "0", "#38bdf8"),
            ("reachable", "متاح عبر TCP", "0", "#10b981"),
            ("unreachable", "غير متاح / معطل", "0", "#ef4444"),
            ("ping", "أفضل زمن استجابة", "-- ms", "#fbbf24"),
            ("updated", "آخر تحديث للقائمة", "لم يتم", "#94a3b8"),
        ]

        for key, label, val, color in card_defs:
            card = tk.Frame(stats_frame, bg="#1e293b", padx=14, pady=10, relief="flat", highlightthickness=1, highlightbackground="#334155")
            card.pack(side="right" if self.current_lang == "ar" else "left", expand=True, fill="both", padx=6)

            lbl = tk.Label(card, text=label, font=("Segoe UI", 9), fg="#94a3b8", bg="#1e293b")
            lbl.pack(anchor="center")

            val_lbl = tk.Label(card, text=val, font=("Segoe UI", 14, "bold"), fg=color, bg="#1e293b")
            val_lbl.pack(anchor="center")
            self.stat_cards[key] = val_lbl

        # Toolbar
        toolbar = tk.Frame(self.root, bg="#0b1120", padx=20, pady=8)
        toolbar.pack(fill="x")

        self.btn_fetch = tk.Button(toolbar, text="📥 تحديث من GitHub", font=("Segoe UI", 10, "bold"), bg="#0284c7", fg="#ffffff",
                                   bd=0, padx=14, pady=8, cursor="hand2", command=self._fetch_from_github)
        self.btn_fetch.pack(side="right" if self.current_lang == "ar" else "left", padx=4)

        self.btn_validate = tk.Button(toolbar, text="🔄 فحص الكل (TCP)", font=("Segoe UI", 10, "bold"), bg="#1e3a8a", fg="#38bdf8",
                                      bd=0, padx=14, pady=8, cursor="hand2", command=self._validate_all)
        self.btn_validate.pack(side="right" if self.current_lang == "ar" else "left", padx=4)

        self.btn_fastest = tk.Button(toolbar, text="⚡ اتصال بالأسرع", font=("Segoe UI", 10, "bold"), bg="#059669", fg="#ffffff",
                                     bd=0, padx=14, pady=8, cursor="hand2", command=self._connect_fastest)
        self.btn_fastest.pack(side="right" if self.current_lang == "ar" else "left", padx=4)

        # Search box
        search_frame = tk.Frame(toolbar, bg="#1e293b", padx=8, pady=4, highlightthickness=1, highlightbackground="#334155")
        search_frame.pack(side="left" if self.current_lang == "ar" else "right", padx=6)

        search_lbl = tk.Label(search_frame, text="🔍", bg="#1e293b", fg="#94a3b8")
        search_lbl.pack(side="left", padx=2)

        self.search_entry = tk.Entry(search_frame, bg="#1e293b", fg="#ffffff", insertbackground="#ffffff", bd=0, font=("Segoe UI", 10), width=24)
        self.search_entry.pack(side="left", padx=4)
        self.search_entry.bind("<KeyRelease>", self._on_search)

        # Notice Banner
        notice = tk.Frame(self.root, bg="#0f172a", padx=16, pady=6, highlightthickness=1, highlightbackground="#0284c7")
        notice.pack(fill="x", padx=20, pady=4)
        notice_text = "ℹ️ ملاحظة فنية: نجاح اختبار TCP يؤكد إمكانية الوصول إلى المنفذ. تفعيل اتصال MTProto الفعلي يتم بموافقتك في نافذة Telegram Desktop."
        tk.Label(notice, text=notice_text, font=("Segoe UI", 9), fg="#94a3b8", bg="#0f172a").pack(side="right" if self.current_lang == "ar" else "left")

        # Table Frame
        table_frame = tk.Frame(self.root, bg="#0b1120", padx=20, pady=8)
        table_frame.pack(fill="both", expand=True)

        columns = ("fav", "server", "port", "secret", "ping", "status", "last_check")
        self.tree = ttk.Treeview(table_frame, columns=columns, show="headings", selectmode="browse")

        self.tree.heading("fav", text="⭐")
        self.tree.heading("server", text="الخادم (Server)")
        self.tree.heading("port", text="المنفذ")
        self.tree.heading("secret", text="المفتاح (Secret)")
        self.tree.heading("ping", text="الاستجابة")
        self.tree.heading("status", text="حالة الفحص")
        self.tree.heading("last_check", text="آخر فحص")

        self.tree.column("fav", width=40, anchor="center")
        self.tree.column("server", width=260, anchor="w")
        self.tree.column("port", width=80, anchor="center")
        self.tree.column("secret", width=220, anchor="w")
        self.tree.column("ping", width=100, anchor="center")
        self.tree.column("status", width=150, anchor="center")
        self.tree.column("last_check", width=140, anchor="center")

        scrollbar = ttk.Scrollbar(table_frame, orient="vertical", command=self.tree.yview)
        self.tree.configure(yscrollcommand=scrollbar.set)

        scrollbar.pack(side="left" if self.current_lang == "ar" else "right", fill="y")
        self.tree.pack(fill="both", expand=True)

        # Table Actions Footer
        action_footer = tk.Frame(self.root, bg="#0f172a", padx=20, pady=10)
        action_footer.pack(fill="x", side="bottom")

        btn_connect = tk.Button(action_footer, text="✈️ اتصال عبر Telegram", font=("Segoe UI", 10, "bold"), bg="#0284c7", fg="#ffffff",
                                bd=0, padx=14, pady=6, cursor="hand2", command=self._connect_selected)
        btn_connect.pack(side="right" if self.current_lang == "ar" else "left", padx=4)

        btn_copy = tk.Button(action_footer, text="📋 نسخ الرابط كاملاً", font=("Segoe UI", 9, "bold"), bg="#1e293b", fg="#f8fafc",
                             bd=0, padx=12, pady=6, cursor="hand2", command=self._copy_selected_link)
        btn_copy.pack(side="right" if self.current_lang == "ar" else "left", padx=4)

        btn_share = tk.Button(action_footer, text="🔗 مشاركة وتصدير", font=("Segoe UI", 9, "bold"), bg="#1e293b", fg="#f8fafc",
                              bd=0, padx=12, pady=6, cursor="hand2", command=self._share_selected)
        btn_share.pack(side="right" if self.current_lang == "ar" else "left", padx=4)

        btn_test_one = tk.Button(action_footer, text="🔄 فحص هذا البروكسي", font=("Segoe UI", 9, "bold"), bg="#1e293b", fg="#f8fafc",
                                 bd=0, padx=12, pady=6, cursor="hand2", command=self._test_selected)
        btn_test_one.pack(side="right" if self.current_lang == "ar" else "left", padx=4)

        btn_del = tk.Button(action_footer, text="🗑️ حذف", font=("Segoe UI", 9, "bold"), bg="#991b1b", fg="#fecaca",
                            bd=0, padx=12, pady=6, cursor="hand2", command=self._delete_selected)
        btn_del.pack(side="left" if self.current_lang == "ar" else "right", padx=4)

        self.tree.bind("<Double-1>", lambda e: self._connect_selected())

    def _load_local_data(self):
        self.proxies = self.db.get_all_proxies()
        if not self.proxies:
            self._fetch_from_github()
        else:
            self._render_table()
            self._update_stats()

    def _render_table(self):
        for item in self.tree.get_children():
            self.tree.delete(item)

        q = self.search_query.lower()
        filtered = [
            p for p in self.proxies
            if not q or q in p['server'].lower() or q in str(p['port'])
        ]

        # Sorting: ping first
        filtered.sort(key=lambda p: (
            p['ping_ms'] if p['ping_ms'] is not None and p['status'] == 'TcpReachable' else 999999
        ))

        for p in filtered:
            status_text = {
                'TcpReachable': '🟢 قابل للوصول عبر TCP',
                'Unreachable': '🔴 غير قابل للوصول',
                'Timeout': '🟠 انتهت المهلة',
                'DnsFailed': '🟣 فشل DNS',
                'Unchecked': '⚪ لم يتم الفحص'
            }.get(p['status'], p['status'])

            ping_text = f"{p['ping_ms']} ms" if p['ping_ms'] is not None else "--"
            secret_disp = mask_secret(p['secret'])
            fav_disp = "⭐" if p['is_favorite'] else "☆"

            self.tree.insert("", "end", iid=p['id'], values=(
                fav_disp, p['server'], p['port'], secret_disp, ping_text, status_text, p['last_check'] or "--"
            ))

    def _update_stats(self):
        total = len(self.proxies)
        reachable = sum(1 for p in self.proxies if p['status'] == 'TcpReachable')
        unreachable = sum(1 for p in self.proxies if p['status'] in ('Unreachable', 'Timeout', 'DnsFailed'))
        pings = [p['ping_ms'] for p in self.proxies if p['status'] == 'TcpReachable' and p['ping_ms'] is not None]
        best_ping = f"{min(pings)} ms" if pings else "-- ms"
        last_update = self.db.get_setting("last_update_timestamp", "لم يتم")

        self.stat_cards['total'].config(text=str(total))
        self.stat_cards['reachable'].config(text=str(reachable))
        self.stat_cards['unreachable'].config(text=str(unreachable))
        self.stat_cards['ping'].config(text=best_ping)
        self.stat_cards['updated'].config(text=last_update)

    def _fetch_from_github(self):
        def task():
            log_event("INFO", "FETCHER", f"Initiating concurrent dual-source sync from GitHub feeds...")
            self.btn_fetch.config(state="disabled", text="جاري التحديث...")
            sources = [
                ("Primary (SoliSpirit)", PRIMARY_SOURCE_URL),
                ("Secondary (mtproto-auto-proxies)", SECONDARY_SOURCE_URL)
            ]
            all_fetched = []
            errors = []

            for name, url in sources:
                try:
                    log_event("INFO", "FETCHER", f"Fetching from {name}: {url}")
                    req = urllib.request.Request(
                        url,
                        headers={'User-Agent': 'MTProxyDesk/1.0.2 (Windows)'}
                    )
                    with urllib.request.urlopen(req, timeout=12) as response:
                        raw_text = response.read().decode('utf-8', errors='ignore')
                    parsed = parse_proxies_text(raw_text, url)
                    log_event("INFO", "FETCHER", f"Received {len(parsed)} proxies from {name}.")
                    all_fetched.extend(parsed)
                except Exception as e:
                    log_event("WARN", "FETCHER", f"Failover active: source {name} encountered {e}")
                    errors.append(f"{name}: {e}")

            # Canonical deduplication
            seen = set()
            deduped = []
            for p in all_fetched:
                key = p.get('id') or f"{p['server'].lower()}:{p['port']}:{p['secret']}"
                if key not in seen:
                    seen.add(key)
                    deduped.append(p)

            if deduped:
                self.db.upsert_proxies(deduped)
                now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
                self.db.set_setting("last_update_timestamp", now_str)
                log_event("INFO", "FETCHER", f"Total unified deduplicated proxies: {len(deduped)}.")
                self.root.after(0, self._on_fetch_complete, True, len(deduped))
            elif errors:
                log_event("ERROR", "FETCHER", f"All sources failed: {errors}")
                self.root.after(0, self._on_fetch_complete, False, " | ".join(errors))
            else:
                self.root.after(0, self._on_fetch_complete, True, 0)

        threading.Thread(target=task, daemon=True).start()

    def _on_fetch_complete(self, success, result):
        self.btn_fetch.config(state="normal", text="📥 تحديث من GitHub")
        if success:
            self.proxies = self.db.get_all_proxies()
            self._render_table()
            self._update_stats()
            messagebox.showinfo("نجاح", f"تم تحديث القائمة بنجاح! تم استلام {result} بروكسي.")
        else:
            messagebox.showwarning("تنبيه الاتصال", f"تعذر الاتصال بـ GitHub: {result}\nتم الإبقاء على القائمة المحلية المحفوظة.")

    def _validate_all(self):
        if self.is_validating:
            return
        self.is_validating = True
        self.btn_validate.config(state="disabled", text="جاري الفحص...")

        def task():
            log_event("INFO", "VALIDATOR", f"Starting batch validation of {len(self.proxies)} proxies.")
            with ThreadPoolExecutor(max_workers=30) as executor:
                results = list(executor.map(validate_proxy, self.proxies))

            now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
            for proxy_id, status, ping_ms in results:
                self.db.update_proxy_status(proxy_id, status, ping_ms, now_str)

            self.root.after(0, self._on_validate_complete)

        threading.Thread(target=task, daemon=True).start()

    def _on_validate_complete(self):
        self.is_validating = False
        self.btn_validate.config(state="normal", text="🔄 فحص الكل (TCP)")
        self.proxies = self.db.get_all_proxies()
        self._render_table()
        self._update_stats()
        log_event("INFO", "VALIDATOR", "Batch validation completed.")
        messagebox.showinfo("اكتمل الفحص", "اكتمل فحص جميع البروكسيات بنجاح.")

    def _connect_selected(self):
        selected = self.tree.selection()
        if not selected:
            messagebox.showwarning("تنبيه", "يرجى اختيار بروكسي من الجدول أولاً.")
            return

        proxy_id = selected[0]
        proxy = next((p for p in self.proxies if p['id'] == proxy_id), None)
        if not proxy:
            return

        res = launch_telegram(proxy['server'], proxy['port'], proxy['secret'])
        messagebox.showinfo(
            "اتصال Telegram",
            f"تم إرسال رابط البروكسي لتشغيل Telegram Desktop.\n\nالخادم: {proxy['server']}:{proxy['port']}\n\nيرجى الضغط على «تمكين» أو «Enable Proxy» داخل تطبيق Telegram لتأكيد الاستخدام."
        )

    def _connect_fastest(self):
        reachable = [p for p in self.proxies if p['status'] == 'TcpReachable' and p['ping_ms'] is not None]
        if not reachable:
            messagebox.showwarning("تنبيه", "لا توجد بروكسيات مفحوصة ومتاحة حالياً. يرجى الضغط على «فحص الكل» أولاً.")
            return

        fastest = min(reachable, key=lambda p: p['ping_ms'])
        self.tree.selection_set(fastest['id'])
        self.tree.see(fastest['id'])
        res = launch_telegram(fastest['server'], fastest['port'], fastest['secret'])
        messagebox.showinfo(
            "أسرع بروكسي متصل",
            f"تم اختيار أسرع بروكسي ({fastest['ping_ms']} ms):\n\n{fastest['server']}:{fastest['port']}\n\nتم تشغيل Telegram Desktop للموافقة والاتصال."
        )

    def _copy_selected_link(self):
        selected = self.tree.selection()
        if not selected:
            messagebox.showwarning("تنبيه", "يرجى اختيار بروكسي أولاً.")
            return

        proxy = next((p for p in self.proxies if p['id'] == selected[0]), None)
        if proxy:
            link = f"tg://proxy?server={proxy['server']}&port={proxy['port']}&secret={proxy['secret']}"
            self.root.clipboard_clear()
            self.root.clipboard_append(link)
            messagebox.showinfo("تم النسخ", "تم نسخ رابط Telegram المباشر إلى الحافظة بنجاح!")

    def _test_selected(self):
        selected = self.tree.selection()
        if not selected:
            return
        proxy = next((p for p in self.proxies if p['id'] == selected[0]), None)
        if not proxy:
            return

        def task():
            _, status, ping = validate_proxy(proxy)
            now_str = datetime.now().strftime("%Y-%m-%d %H:%M:%S")
            self.db.update_proxy_status(proxy['id'], status, ping, now_str)
            self.root.after(0, self._load_local_data)

        threading.Thread(target=task, daemon=True).start()

    def _delete_selected(self):
        selected = self.tree.selection()
        if not selected:
            return
        if messagebox.askyesno("تأكيد الحذف", "هل أنت متأكد من حذف هذا البروكسي من القائمة المحلية؟"):
            self.db.delete_proxy(selected[0])
            self._load_local_data()

    def _share_selected(self):
        selected = self.tree.selection()
        if not selected:
            messagebox.showwarning("تنبيه", "يرجى اختيار بروكسي أولاً.")
            return
        p = next((p for p in self.proxies if p['id'] == selected[0]), None)
        if not p:
            return

        tg_link = f"tg://proxy?server={p['server']}&port={p['port']}&secret={p['secret']}"
        https_link = f"https://t.me/proxy?server={p['server']}&port={p['port']}&secret={p['secret']}"

        win = tk.Toplevel(self.root)
        win.title("مشاركة بيانات البروكسي")
        win.geometry("520x380")
        win.configure(bg="#0f172a")

        tk.Label(win, text="⚠️ تحذير أمني: مشاركة المفتاح تتيح للآخرين الاتصال عبر هذا الخادم.",
                 fg="#fde68a", bg="#78350f", padx=10, pady=6, font=("Segoe UI", 9, "bold")).pack(fill="x", padx=16, pady=12)

        tk.Label(win, text="رابط Telegram المباشر (tg://):", fg="#94a3b8", bg="#0f172a", font=("Segoe UI", 9)).pack(anchor="w", padx=16)
        e1 = tk.Entry(win, width=55, font=("Consolas", 9))
        e1.insert(0, tg_link)
        e1.pack(padx=16, pady=4)

        tk.Label(win, text="رابط الويب البديل (https://t.me/):", fg="#94a3b8", bg="#0f172a", font=("Segoe UI", 9)).pack(anchor="w", padx=16)
        e2 = tk.Entry(win, width=55, font=("Consolas", 9))
        e2.insert(0, https_link)
        e2.pack(padx=16, pady=4)

        def save_txt():
            path = filedialog.asksaveasfilename(defaultextension=".txt", initialfile=f"MTProxy_{p['server']}_{p['port']}.txt")
            if path:
                with open(path, "w", encoding="utf-8") as f:
                    f.write(f"MTProxy Desk Export\nServer: {p['server']}\nPort: {p['port']}\nSecret: {p['secret']}\nLink: {tg_link}\n")
                messagebox.showinfo("تم الحفظ", "تم حفظ ملف البروكسي بنجاح.")

        tk.Button(win, text="💾 حفظ كملف TXT", bg="#0284c7", fg="#ffffff", bd=0, padx=12, pady=6, command=save_txt).pack(pady=12)

    def _show_diagnostics(self):
        win = tk.Toplevel(self.root)
        win.title("سجل التشخيص والأخطاء (Diagnostics)")
        win.geometry("700x480")
        win.configure(bg="#0b1120")

        text = tk.Text(win, bg="#030712", fg="#38bdf8", font=("Consolas", 9), padx=10, pady=10)
        text.pack(fill="both", expand=True, padx=12, pady=12)

        if os.path.exists(LOG_PATH):
            with open(LOG_PATH, "r", encoding="utf-8") as f:
                content = f.read()
                text.insert("1.0", content if content else "لا توجد سجلات بعد.")
        else:
            text.insert("1.0", "لم يتم إنشاء ملف سجلات بعد.")

    def _on_search(self, event):
        self.search_query = self.search_entry.get().strip()
        self._render_table()

    def _toggle_language(self):
        self.current_lang = "en" if self.current_lang == "ar" else "ar"
        messagebox.showinfo("اللغة", f"تم التبديل إلى: {self.current_lang.upper()}")

    def _auto_update_loop(self):
        while True:
            # Sleep 30 minutes (1800 seconds)
            time.sleep(1800)
            log_event("INFO", "AUTO_UPDATER", "30-minute periodic update triggered.")
            self._fetch_from_github()

if __name__ == "__main__":
    root = tk.Tk()
    app = MTProxyDeskApp(root)
    root.mainloop()
