/**
 * MTProxy Desk – Telegram Proxy Manager
 * Professional Controller (Optimized Table, Right-Click Context Menu, Details Modal, 30m Auto-Sync)
 * Developer: Mohammed Shaheed Al-Aajeebi
 */

// Application State
let allProxies = [];
let currentView = 'dashboard';
let currentFilter = 'all';
let currentSort = 'ping';
let searchQuery = '';
let currentLang = 'en';
let isTestingAll = false;
let activeShareProxy = null;
let activeContextProxy = null;
let appSettings = null;

// Complete Dictionary
const i18n = {
  en: {
    appTitle: "MTProxy Desk",
    appVersion: "v1.0.2",
    devTag: "Dev: Mohammed Shaheed Al-Aajeebi",
    langBtn: "العربية",

    navDashboard: "All Proxies",
    navFastest: "Fastest",
    navFavorites: "Favorites",
    navShare: "Share & QR",
    navDiagnostics: "Diagnostics",
    navSettings: "Settings",

    viewTitleDashboard: "Server Dashboard",
    viewSubDashboard: "Live Telegram proxies fetched automatically from GitHub (30m sync)",
    viewTitleFastest: "⚡ Lowest Latency Verified Proxies",
    viewSubFastest: "Fastest responding MTProto servers for instantaneous connection",
    viewTitleFavorites: "⭐ Bookmarked Proxies",
    viewSubFavorites: "Pinned proxy servers for immediate 1-click launch",
    viewTitleShare: "🔗 Share & QR Studio",
    viewSubShare: "Direct protocol URIs, QR scan codes, and TXT configurations",
    viewTitleDiagnostics: "🩺 Runtime Diagnostics & Live Engine Logs",
    viewSubDiagnostics: "Tokio worker thread pool, SQLite events, and network status",
    viewTitleSettings: "⚙️ Settings & Project Repository",
    viewSubSettings: "Configure 30-minute background sync, concurrency, and custom feeds",

    btnFetch: "Update",
    btnValidate: "Retest All",
    btnFastest: "Connect Fastest",

    searchPlaceholder: "Filter by IP, port or secret...",
    chipAll: "All",
    chipReachable: "🟢 Reachable",
    chipFavorites: "⭐ Favorites",
    chipUnreachable: "🔴 Dead",

    thConnect: "Connect",
    thShare: "Share",
    thServer: "Server Host",
    thPort: "Port",
    thPing: "Ping",
    thInfo: "Info",

    btnConnectTg: "Connect",

    copied: "Copied to clipboard!",
    copiedLink: "Proxy link copied!",
    copiedServer: "Server address copied!",
    copiedPort: "Port number copied!",
    copiedSecret: "Secret key copied!",
    fetching: "Fetching latest proxies from GitHub...",
    validating: "Testing proxies concurrently...",
    validationDone: "Proxy check completed.",
    settingsSaved: "Settings saved successfully!",
    proxyDeleted: "Proxy removed."
  },

  ar: {
    appTitle: "MTProxy Desk",
    appVersion: "v1.0.2",
    devTag: "المطور: محمد شهيد الاعاجيبي",
    langBtn: "English",

    navDashboard: "كل البروكسيات",
    navFastest: "الأسرع",
    navFavorites: "المفضلة",
    navShare: "المشاركة و QR",
    navDiagnostics: "التشخيص",
    navSettings: "الإعدادات",

    viewTitleDashboard: "لوحة خوادم Telegram",
    viewSubDashboard: "بروكسيات MTProto الحية مع التحديث التلقائي كل 30 دقيقة",
    viewTitleFastest: "⚡ البروكسيات الأسرع استجابة",
    viewSubFastest: "أفضل الخوادم سرعة وأقلها تأخيراً للاتصال الفوري",
    viewTitleFavorites: "⭐ البروكسيات المفضلة",
    viewSubFavorites: "الخوادم المثبتة لديك للوصول السريع والربط بنقرة واحدة",
    viewTitleShare: "🔗 استوديو المشاركة والباركود",
    viewSubShare: "روابط Telegram المباشرة ورموز QR وتصدير التكوينات",
    viewTitleDiagnostics: "🩺 سجل التشخيص وفحص العمليات الحية",
    viewSubDiagnostics: "متابعة خيوط Tokio ومحرك SQLite والأحداث المباشرة",
    viewTitleSettings: "⚙️ الإعدادات ومستودع المشروع",
    viewSubSettings: "ضبط التحديث التلقائي كل 30 دقيقة والأداء وإدارة المصادر",

    btnFetch: "تحديث",
    btnValidate: "فحص شامل",
    btnFastest: "أسرع بروكسي",

    searchPlaceholder: "فلترة بالخادم أو المنفذ أو المفتاح...",
    chipAll: "الكل",
    chipReachable: "🟢 متاح",
    chipFavorites: "⭐ المفضلة",
    chipUnreachable: "🔴 معطل",

    thConnect: "اتصال",
    thShare: "مشاركة",
    thServer: "عنوان الخادم",
    thPort: "المنفذ",
    thPing: "الاستجابة",
    thInfo: "تفاصيل",

    btnConnectTg: "اتصال",

    copied: "تم النسخ بنجاح!",
    copiedLink: "تم نسخ رابط البروكسي!",
    copiedServer: "تم نسخ الخادم!",
    copiedPort: "تم نسخ المنفذ!",
    copiedSecret: "تم نسخ المفتاح!",
    fetching: "جاري جلب أحدث قائمة من GitHub...",
    validating: "جاري فحص البروكسيات بالتوازي...",
    validationDone: "اكتمل الفحص بنجاح.",
    settingsSaved: "تم حفظ الإعدادات بنجاح!",
    proxyDeleted: "تم حذف البروكسي."
  }
};

// ==========================================
// TAURI IPC RESOLVER
// ==========================================
async function invokeBackend(cmd, args = {}) {
  let invoker = null;
  if (window.__TAURI__ && window.__TAURI__.core && window.__TAURI__.core.invoke) {
    invoker = window.__TAURI__.core.invoke;
  } else if (window.__TAURI_INTERNALS__ && window.__TAURI_INTERNALS__.invoke) {
    invoker = window.__TAURI_INTERNALS__.invoke;
  }

  if (invoker) {
    try {
      return await invoker(cmd, args);
    } catch (err) {
      console.error(`[IPC Error: ${cmd}]`, err);
      showToast(`Error: ${err}`);
      throw err;
    }
  }

  console.warn(`[No Tauri IPC available for ${cmd}]`, args);
  return null;
}

// Toast
function showToast(message, duration = 2400) {
  const toast = document.getElementById('app-toast');
  const toastText = document.getElementById('toast-text');
  toastText.textContent = message;
  toast.classList.remove('hidden');

  clearTimeout(window._toastTimer);
  window._toastTimer = setTimeout(() => {
    toast.classList.add('hidden');
  }, duration);
}

function copyToClipboard(text, successMsg) {
  navigator.clipboard.writeText(text).then(() => {
    showToast(successMsg || i18n[currentLang].copied);
  }).catch(() => {
    showToast("Copy failed");
  });
}

// View Switching
function switchView(viewName) {
  currentView = viewName;

  document.querySelectorAll('.nav-btn').forEach(btn => {
    btn.classList.toggle('active', btn.dataset.view === viewName);
  });

  document.querySelectorAll('.view').forEach(sec => {
    sec.classList.toggle('active', sec.id === `view-${viewName}`);
  });

  const t = i18n[currentLang];
  const titleEl = document.getElementById('view-title');
  const subEl = document.getElementById('view-subtitle');

  switch (viewName) {
    case 'dashboard':
      titleEl.textContent = t.viewTitleDashboard;
      subEl.textContent = t.viewSubDashboard;
      renderProxiesTable();
      break;
    case 'fastest':
      titleEl.textContent = t.viewTitleFastest;
      subEl.textContent = t.viewSubFastest;
      renderFastestView();
      break;
    case 'favorites':
      titleEl.textContent = t.viewTitleFavorites;
      subEl.textContent = t.viewSubFavorites;
      renderFavoritesView();
      break;
    case 'share':
      titleEl.textContent = t.viewTitleShare;
      subEl.textContent = t.viewSubShare;
      renderShareStudio();
      break;
    case 'diagnostics':
      titleEl.textContent = t.viewTitleDiagnostics;
      subEl.textContent = t.viewSubDiagnostics;
      refreshDiagnosticsLogs();
      break;
    case 'settings':
      titleEl.textContent = t.viewTitleSettings;
      subEl.textContent = t.viewSubSettings;
      loadSettingsView();
      break;
  }
}

// Language Toggle
function setLanguage(lang) {
  currentLang = lang;
  const isRtl = lang === 'ar';
  document.documentElement.dir = isRtl ? 'rtl' : 'ltr';
  document.documentElement.lang = lang;

  const t = i18n[lang];
  document.getElementById('lang-btn-text').textContent = t.langBtn;

  document.querySelector('#nav-btn-dashboard .nav-text').textContent = t.navDashboard;
  document.querySelector('#nav-btn-fastest .nav-text').textContent = t.navFastest;
  document.querySelector('#nav-btn-favorites .nav-text').textContent = t.navFavorites;
  document.querySelector('#nav-btn-share .nav-text').textContent = t.navShare;
  document.querySelector('#nav-btn-diagnostics .nav-text').textContent = t.navDiagnostics;
  document.querySelector('#nav-btn-settings .nav-text').textContent = t.navSettings;

  document.getElementById('btn-fetch').innerHTML = `<span>📥</span> ${t.btnFetch}`;
  document.getElementById('btn-validate-all').innerHTML = `<span>🔄</span> ${t.btnValidate}`;
  document.getElementById('btn-connect-fastest').innerHTML = `<span>⚡</span> ${t.btnFastest}`;

  document.getElementById('input-search').placeholder = t.searchPlaceholder;
  document.getElementById('chip-all').textContent = t.chipAll;
  document.getElementById('chip-reachable').textContent = t.chipReachable;
  document.getElementById('chip-favorites').textContent = t.chipFavorites;
  document.getElementById('chip-unreachable').textContent = t.chipUnreachable;

  document.getElementById('th-connect').textContent = t.thConnect;
  document.getElementById('th-share').textContent = t.thShare;
  document.getElementById('th-server').textContent = t.thServer;
  document.getElementById('th-port').textContent = t.thPort;
  document.getElementById('th-ping').textContent = t.thPing;
  document.getElementById('th-info').textContent = t.thInfo;

  switchView(currentView);
}

function getPingBadge(ping) {
  if (ping === null || ping === undefined) return `<span class="badge-ping ping-none">--</span>`;
  if (ping < 180) return `<span class="badge-ping ping-fast">⚡ ${ping}ms</span>`;
  if (ping < 450) return `<span class="badge-ping ping-med">${ping}ms</span>`;
  return `<span class="badge-ping ping-slow">${ping}ms</span>`;
}

// Stats sync
async function refreshStats() {
  try {
    const stats = await invokeBackend('get_stats');
    if (!stats) return;

    document.getElementById('val-stat-total').textContent = stats.total;
    document.getElementById('val-stat-reachable').textContent = stats.tcp_reachable;
    document.getElementById('val-stat-unreachable').textContent = stats.unreachable + stats.timeout + stats.dns_failed;
    document.getElementById('val-stat-ping').textContent = stats.best_ping_ms !== null ? `${stats.best_ping_ms} ms` : '-- ms';
    document.getElementById('lbl-last-sync').textContent = stats.last_update ? stats.last_update.split(' ')[1] || stats.last_update : 'Synced';

    document.getElementById('badge-total-count').textContent = stats.total;
    document.getElementById('badge-fast-count').textContent = stats.tcp_reachable;
    document.getElementById('badge-fav-count').textContent = allProxies.filter(p => p.is_favorite).length;

    isTestingAll = stats.is_testing;
  } catch (e) {
    console.error("Stats refresh error", e);
  }
}

// Filtering
function getFilteredProxies() {
  let list = [...allProxies];

  if (searchQuery.trim()) {
    const q = searchQuery.trim().toLowerCase();
    list = list.filter(p =>
      p.server.toLowerCase().includes(q) ||
      p.port.toString().includes(q) ||
      p.secret.toLowerCase().includes(q)
    );
  }

  if (currentFilter === 'reachable') {
    list = list.filter(p => p.status === 'TcpReachable');
  } else if (currentFilter === 'favorites') {
    list = list.filter(p => p.is_favorite);
  } else if (currentFilter === 'unreachable') {
    list = list.filter(p => p.status === 'Unreachable' || p.status === 'Timeout' || p.status === 'DnsFailed');
  }

  if (currentSort === 'ping') {
    list.sort((a, b) => {
      const pingA = a.ping_ms !== null ? a.ping_ms : 999999;
      const pingB = b.ping_ms !== null ? b.ping_ms : 999999;
      return pingA - pingB;
    });
  } else if (currentSort === 'port') {
    list.sort((a, b) => a.port - b.port);
  } else if (currentSort === 'server') {
    list.sort((a, b) => a.server.localeCompare(b.server));
  }

  return list;
}

// ==========================================
// RENDER COMPACT TABLE (NO HORIZONTAL SCROLL)
// Connect & Share positioned BEFORE Server
// ==========================================
function renderProxiesTable() {
  const tbody = document.getElementById('proxies-tbody');
  const filtered = getFilteredProxies();
  const t = i18n[currentLang];

  document.getElementById('search-counter').textContent = `${filtered.length}`;

  if (filtered.length === 0) {
    tbody.innerHTML = `<tr><td colspan="7" class="empty-cell"><span>No proxies match search or filter.</span></td></tr>`;
    return;
  }

  let html = '';
  filtered.forEach(p => {
    const favClass = p.is_favorite ? 'active' : '';

    html += `
      <tr data-proxy-id="${p.id}" class="proxy-row">
        <td style="text-align: center;">
          <button class="star-btn ${favClass}" onclick="handleToggleFavorite('${p.id}', event)" title="Favorite">⭐</button>
        </td>
        <td style="text-align: center;">
          <button class="btn-connect-cell" onclick="handleLaunchTelegram('${p.id}', event)" title="Direct Connect in Telegram">
            <span>✈️</span> ${t.btnConnectTg}
          </button>
        </td>
        <td style="text-align: center;">
          <button class="btn-share-cell" onclick="handleOpenShare('${p.id}', event)" title="Share & QR Code">
            <span>🔗</span>
          </button>
        </td>
        <td>
          <div class="cell-host" title="Right-click for options / Click to copy">
            <span>${p.server}</span>
            <button class="icon-sub" onclick="copyToClipboard('${p.server}', '${t.copiedServer}', event)" title="Copy Host">📋</button>
          </div>
        </td>
        <td style="text-align: center;">
          <span class="port-badge" onclick="copyToClipboard('${p.port}', '${t.copiedPort}', event)" title="Click to copy Port: ${p.port}">${p.port}</span>
        </td>
        <td style="text-align: center;">
          ${getPingBadge(p.ping_ms)}
        </td>
        <td style="text-align: center;">
          <button class="btn-info-cell" onclick="openDetailsModal('${p.id}', event)" title="View Full Details">
            ℹ️
          </button>
        </td>
      </tr>
    `;
  });

  tbody.innerHTML = html;

  // Attach context menu listener on each row
  document.querySelectorAll('.proxy-row').forEach(row => {
    row.addEventListener('contextmenu', (e) => {
      e.preventDefault();
      const proxyId = row.dataset.proxyId;
      const found = allProxies.find(x => x.id === proxyId);
      if (found) {
        openContextMenu(e, found);
      }
    });
  });
}

// ==========================================
// RIGHT-CLICK CONTEXT MENU CONTROLLER
// ==========================================
function openContextMenu(e, proxy) {
  activeContextProxy = proxy;
  const menu = document.getElementById('context-menu');

  // Highlight active row in table
  document.querySelectorAll('.proxy-row').forEach(r => r.classList.remove('active-row'));
  const targetRow = document.querySelector(`[data-proxy-id="${proxy.id}"]`);
  if (targetRow) targetRow.classList.add('active-row');

  menu.style.display = 'block';

  // Position context menu constrained to window bounds
  let x = e.clientX;
  let y = e.clientY;
  const menuWidth = 210;
  const menuHeight = 320;

  if (x + menuWidth > window.innerWidth) x = window.innerWidth - menuWidth - 10;
  if (y + menuHeight > window.innerHeight) y = window.innerHeight - menuHeight - 10;

  menu.style.left = `${Math.max(10, x)}px`;
  menu.style.top = `${Math.max(10, y)}px`;
}

function hideContextMenu() {
  const menu = document.getElementById('context-menu');
  menu.style.display = 'none';
  document.querySelectorAll('.proxy-row').forEach(r => r.classList.remove('active-row'));
}

// ==========================================
// QUICK DETAILS MODAL
// ==========================================
function openDetailsModal(proxyId, e) {
  if (e) e.stopPropagation();
  const p = allProxies.find(x => x.id === proxyId);
  if (!p) return;

  const modal = document.getElementById('modal-details');
  const body = document.getElementById('modal-details-body');

  const tgLink = `tg://proxy?server=${encodeURIComponent(p.server)}&port=${p.port}&secret=${encodeURIComponent(p.secret)}`;
  const httpLink = `https://t.me/proxy?server=${encodeURIComponent(p.server)}&port=${p.port}&secret=${encodeURIComponent(p.secret)}`;

  let sourceBadge = "🌐 SoliSpirit/mtproto (GitHub Raw)";
  if (p.source_url && p.source_url.includes("mtproto-auto-proxies")) {
    sourceBadge = "⚡ mtproto-auto-proxies (Telegram /s/ Channel Feed)";
  } else if (p.source_url && !p.source_url.includes("SoliSpirit")) {
    sourceBadge = `📡 Custom Feed: ${p.source_url}`;
  }

  body.innerHTML = `
    <div class="detail-row">
      <span class="detail-label">Server Host / IP:</span>
      <div class="detail-value-box">
        <span>${p.server}</span>
        <button class="btn btn-secondary btn-xs" onclick="copyToClipboard('${p.server}')">📋 Copy</button>
      </div>
    </div>

    <div class="detail-row">
      <span class="detail-label">Port Number:</span>
      <div class="detail-value-box">
        <span style="color: #38bdf8; font-weight: 700;">${p.port}</span>
        <button class="btn btn-secondary btn-xs" onclick="copyToClipboard('${p.port}')">📋 Copy</button>
      </div>
    </div>

    <div class="detail-row">
      <span class="detail-label">Proxy Secret:</span>
      <div class="detail-value-box">
        <span style="font-size: 11px; word-break: break-all;">${p.secret}</span>
        <button class="btn btn-secondary btn-xs" onclick="copyToClipboard('${p.secret}')">📋 Copy</button>
      </div>
    </div>

    <div style="display: flex; gap: 8px;">
      <div class="detail-row" style="flex: 1;">
        <span class="detail-label">Latency / Ping:</span>
        <div class="detail-value-box">
          ${getPingBadge(p.ping_ms)}
        </div>
      </div>
      <div class="detail-row" style="flex: 1;">
        <span class="detail-label">Status:</span>
        <div class="detail-value-box">
          <span style="font-size: 11px;">${p.status}</span>
        </div>
      </div>
    </div>

    <div class="detail-row">
      <span class="detail-label">Origin Feed / المصدر المستلم منه:</span>
      <div class="detail-value-box" style="background: rgba(30, 41, 59, 0.85); border-color: rgba(56, 189, 248, 0.4);">
        <span style="color: #38bdf8; font-weight: 600; font-size: 11px;">${sourceBadge}</span>
        <button class="btn btn-secondary btn-xs" onclick="copyToClipboard('${p.source_url || 'https://raw.githubusercontent.com/moshraheem-sudo/mtproto-auto-proxies/main/proxies.txt'}')">📋 Copy Feed</button>
      </div>
    </div>

    <div class="detail-row">
      <span class="detail-label">Direct Telegram URI:</span>
      <div class="detail-value-box">
        <input type="text" readonly value="${tgLink}" style="flex: 1; background: transparent; border: none; color: #cbd5e1; font-size: 10.5px; font-family: monospace; outline: none;">
        <button class="btn btn-secondary btn-xs" onclick="copyToClipboard('${tgLink}')">📋 Copy</button>
      </div>
    </div>

    <div class="modal-actions-row">
      <button class="btn btn-success" onclick="handleLaunchTelegram('${p.id}'); closeDetailsModal();">
        <span>✈️</span> Connect in Telegram Desktop
      </button>
      <button class="btn btn-primary" onclick="handleOpenShare('${p.id}'); closeDetailsModal();">
        <span>🔗</span> QR Studio
      </button>
    </div>
  `;

  modal.classList.remove('hidden');
}

function closeDetailsModal() {
  document.getElementById('modal-details').classList.add('hidden');
}

// Fastest View
function renderFastestView() {
  const container = document.getElementById('fastest-grid-cards');
  const t = i18n[currentLang];

  const reachable = allProxies
    .filter(p => p.status === 'TcpReachable' && p.ping_ms !== null)
    .sort((a, b) => a.ping_ms - b.ping_ms)
    .slice(0, 12);

  if (reachable.length === 0) {
    container.innerHTML = `<div class="empty-cell" style="grid-column: 1 / -1;"><span>No tested reachable proxies yet. Click «Retest All».</span></div>`;
    return;
  }

  let html = '';
  reachable.forEach((p, idx) => {
    html += `
      <div class="fast-card">
        <div class="card-top">
          <span class="rank-pill">Rank #${idx + 1}</span>
          <div>
            <span class="ping-display">${p.ping_ms}</span>
            <span class="ping-unit">ms</span>
          </div>
        </div>
        <div class="card-meta">
          <div><strong>Host:</strong> ${p.server}</div>
          <div><strong>Port:</strong> ${p.port}</div>
        </div>
        <button class="btn btn-success btn-sm btn-block" onclick="handleLaunchTelegram('${p.id}')">
          ✈️ ${t.btnConnectTg}
        </button>
      </div>
    `;
  });

  container.innerHTML = html;
}

// Favorites View
function renderFavoritesView() {
  const container = document.getElementById('favorites-container');
  const t = i18n[currentLang];
  const favs = allProxies.filter(p => p.is_favorite);

  if (favs.length === 0) {
    container.innerHTML = `<div class="empty-cell"><span>No favorites yet. Click the star ⭐ on any proxy.</span></div>`;
    return;
  }

  let html = `
    <table class="compact-table">
      <thead>
        <tr>
          <th style="width: 32px; text-align: center;">⭐</th>
          <th style="width: 90px; text-align: center;">Connect</th>
          <th>Server Host</th>
          <th style="width: 70px; text-align: center;">Port</th>
          <th style="width: 80px; text-align: center;">Latency</th>
          <th style="width: 44px; text-align: center;">Info</th>
        </tr>
      </thead>
      <tbody>
  `;

  favs.forEach(p => {
    html += `
      <tr>
        <td style="text-align: center;">
          <button class="star-btn active" onclick="handleToggleFavorite('${p.id}')">⭐</button>
        </td>
        <td style="text-align: center;">
          <button class="btn-connect-cell" onclick="handleLaunchTelegram('${p.id}')">✈️ Connect</button>
        </td>
        <td><div class="cell-host"><span>${p.server}</span></div></td>
        <td style="text-align: center;"><span class="port-badge">${p.port}</span></td>
        <td style="text-align: center;">${getPingBadge(p.ping_ms)}</td>
        <td style="text-align: center;">
          <button class="btn-info-cell" onclick="openDetailsModal('${p.id}')">ℹ️</button>
        </td>
      </tr>
    `;
  });

  html += `</tbody></table>`;
  container.innerHTML = html;
}

// Share Studio
async function renderShareStudio() {
  if (!activeShareProxy && allProxies.length > 0) {
    activeShareProxy = allProxies.find(p => p.status === 'TcpReachable') || allProxies[0];
  }

  if (!activeShareProxy) {
    document.getElementById('share-proxy-summary').textContent = "No proxies available";
    return;
  }

  const p = activeShareProxy;
  document.getElementById('share-proxy-summary').textContent = `${p.server}:${p.port}`;

  const tgLink = `tg://proxy?server=${encodeURIComponent(p.server)}&port=${p.port}&secret=${encodeURIComponent(p.secret)}`;
  const httpLink = `https://t.me/proxy?server=${encodeURIComponent(p.server)}&port=${p.port}&secret=${encodeURIComponent(p.secret)}`;

  document.getElementById('share-tg-link').value = tgLink;
  document.getElementById('share-http-link').value = httpLink;

  const qrBox = document.getElementById('share-qr-svg');
  try {
    const qrSvg = await invokeBackend('generate_qr', { id: p.id });
    if (qrSvg) qrBox.innerHTML = qrSvg;
    else qrBox.innerHTML = `<div class="qr-hint">${tgLink}</div>`;
  } catch (err) {
    qrBox.innerHTML = `<div class="qr-hint">QR generated in Telegram URI</div>`;
  }
}

function handleOpenShare(proxyId, e) {
  if (e) e.stopPropagation();
  const found = allProxies.find(p => p.id === proxyId);
  if (found) {
    activeShareProxy = found;
    switchView('share');
  }
}

// Diagnostics
async function refreshDiagnosticsLogs() {
  const consoleEl = document.getElementById('logs-console');
  try {
    const logs = await invokeBackend('get_diagnostics_logs');
    consoleEl.textContent = logs || "No logs available.";
    consoleEl.scrollTop = consoleEl.scrollHeight;
  } catch (e) {
    consoleEl.textContent = "Error reading logs: " + e;
  }
}

// Settings
async function loadSettingsView() {
  try {
    appSettings = await invokeBackend('get_settings');
    if (appSettings) {
      document.getElementById('set-autoupdate').checked = appSettings.auto_update;
      document.getElementById('set-interval').value = appSettings.update_interval_minutes || 30;
      document.getElementById('set-autostart').checked = appSettings.autostart;
      document.getElementById('set-tray').checked = appSettings.minimize_to_tray;
      document.getElementById('set-autocheck').checked = appSettings.auto_check;
      document.getElementById('set-concurrency').value = appSettings.max_concurrency || 30;
      document.getElementById('set-timeout').value = appSettings.check_timeout_secs || 4;
      document.getElementById('set-tg-path').value = appSettings.custom_telegram_path || '';
    }

    const sources = await invokeBackend('get_sources');
    const container = document.getElementById('sources-container');
    if (sources && sources.length > 0) {
      container.innerHTML = sources.map(s => `
        <div style="display: flex; justify-content: space-between; align-items: center; background: rgba(255,255,255,0.03); padding: 4px 8px; border-radius: 4px;">
          <span style="font-family: monospace; font-size: 10.5px; color: #38bdf8;">${s.url || s}</span>
          <button class="btn btn-danger btn-xs" onclick="handleDeleteSource('${s.url || s}')">Del</button>
        </div>
      `).join('');
    } else {
      container.innerHTML = `<span style="font-size: 10.5px; color: var(--text-dim);">Default: SoliSpirit GitHub Raw</span>`;
    }
  } catch (err) {
    console.error("Settings load error:", err);
  }
}

async function saveAllSettings() {
  const mins = parseInt(document.getElementById('set-interval').value, 10) || 30;
  const updated = {
    auto_update: document.getElementById('set-autoupdate').checked,
    update_interval_minutes: mins,
    update_interval_hours: Math.max(1, Math.round(mins / 60)),
    autostart: document.getElementById('set-autostart').checked,
    minimize_to_tray: document.getElementById('set-tray').checked,
    auto_check: document.getElementById('set-autocheck').checked,
    max_concurrency: parseInt(document.getElementById('set-concurrency').value, 10) || 30,
    check_timeout_secs: parseInt(document.getElementById('set-timeout').value, 10) || 4,
    language: currentLang,
    theme: "dark",
    custom_telegram_path: document.getElementById('set-tg-path').value.trim() || null,
    last_selected_proxy_id: null,
    last_update_timestamp: null,
  };

  try {
    await invokeBackend('update_settings', { settings: updated });
    showToast(i18n[currentLang].settingsSaved);
  } catch (err) {
    showToast("Failed to save settings: " + err);
  }
}

// Handlers
async function handleFetchProxies() {
  showToast(i18n[currentLang].fetching);
  try {
    const list = await invokeBackend('fetch_proxies');
    if (list && list.length > 0) {
      allProxies = list;
    } else {
      allProxies = await invokeBackend('get_proxies') || [];
    }
    await refreshStats();
    renderProxiesTable();
    showToast(`Loaded ${allProxies.length} proxies successfully!`);
  } catch (e) {
    showToast("Failed to fetch proxies from GitHub");
  }
}

async function handleValidateAll() {
  if (isTestingAll) return;
  showToast(i18n[currentLang].validating);
  try {
    await invokeBackend('validate_all_proxies');
    pollValidation();
  } catch (e) {
    showToast("Failed to start batch test");
  }
}

async function pollValidation() {
  const interval = setInterval(async () => {
    await refreshStats();
    const proxies = await invokeBackend('get_proxies');
    if (proxies && proxies.length > 0) {
      allProxies = proxies;
      if (currentView === 'dashboard') renderProxiesTable();
      if (currentView === 'fastest') renderFastestView();
    }
    if (!isTestingAll) {
      clearInterval(interval);
      showToast(i18n[currentLang].validationDone);
    }
  }, 2000);
}

async function handleLaunchTelegram(proxyId, e) {
  if (e) e.stopPropagation();
  try {
    const result = await invokeBackend('connect_telegram', { id: proxyId });
    if (result && !result.success) {
      showToast(result.message_en || result.message_ar || "Could not launch Telegram Desktop");
    } else {
      showToast("Opening Telegram to add proxy...");
    }
  } catch (e) {
    showToast("Telegram launch error");
  }
}

async function handleConnectFastest() {
  showToast("Finding fastest responsive proxy...");
  try {
    const result = await invokeBackend('connect_fastest_telegram');
    if (result && result.success) {
      showToast(result.message_en || "Connected to fastest proxy!");
    } else {
      showToast("No reachable proxy found yet. Run 'Retest All'.");
    }
  } catch (e) {
    showToast("Fastest connect error");
  }
}

async function handleRetestSingle(proxyId, e) {
  if (e) e.stopPropagation();
  try {
    const updated = await invokeBackend('validate_single_proxy', { id: proxyId });
    if (updated) {
      const idx = allProxies.findIndex(p => p.id === proxyId);
      if (idx !== -1) allProxies[idx] = updated;
      renderProxiesTable();
      await refreshStats();
      showToast(`Ping: ${updated.ping_ms ? updated.ping_ms + ' ms' : 'Dead'}`);
    }
  } catch (e) {
    showToast("Failed to test proxy");
  }
}

async function handleToggleFavorite(proxyId, e) {
  if (e) e.stopPropagation();
  try {
    const newFav = await invokeBackend('toggle_favorite', { id: proxyId });
    const p = allProxies.find(x => x.id === proxyId);
    if (p) p.is_favorite = newFav;
    renderProxiesTable();
    await refreshStats();
    showToast(newFav ? "Added to favorites ⭐" : "Removed from favorites");
  } catch (e) {
    showToast("Favorite error");
  }
}

async function handleDeleteProxy(proxyId, e) {
  if (e) e.stopPropagation();
  try {
    await invokeBackend('delete_proxy', { id: proxyId });
    allProxies = allProxies.filter(p => p.id !== proxyId);
    renderProxiesTable();
    await refreshStats();
    showToast(i18n[currentLang].proxyDeleted);
  } catch (e) {
    showToast("Delete error");
  }
}

async function handlePurgeDeadProxies() {
  try {
    const deadCount = allProxies.filter(p => (p.status === 'Unreachable' || p.status === 'Timeout' || p.status === 'DnsFailed') && !p.is_favorite).length;
    if (deadCount === 0) {
      showToast("No dead proxies to clean.");
      return;
    }
    const count = await invokeBackend('clear_dead_proxies');
    allProxies = await invokeBackend('get_proxies');
    renderProxiesTable();
    await refreshStats();
    showToast(`Successfully removed ${count} dead proxies from local storage.`);
  } catch (e) {
    showToast("Error clearing dead proxies: " + e);
  }
}

async function handleDeleteSource(url) {
  try {
    await invokeBackend('remove_source', { url });
    loadSettingsView();
    showToast("Source removed.");
  } catch (e) {
    showToast("Error deleting source");
  }
}

let currentUpdateInfo = null;
let updateActionState = 'idle'; // 'idle', 'downloading', 'ready_to_install', 'installing'

// ==========================================
// SOFTWARE UPDATER (GITHUB RELEASES)
// ==========================================
async function checkAppUpdates(interactive = false) {
  try {
    if (interactive) {
      showToast("Checking GitHub for updates...");
    }
    const res = await invokeBackend('check_app_updates');
    if (!res) return;
    currentUpdateInfo = res;

    const currentTagElem = document.getElementById('updater-current-tag');
    const latestTagElem = document.getElementById('updater-latest-tag');
    const sizeCol = document.getElementById('updater-size-col');
    const sizeElem = document.getElementById('updater-file-size');
    const statusBadge = document.getElementById('updater-status-badge');
    const notesElem = document.getElementById('updater-notes');
    const startBtn = document.getElementById('btn-start-install-update');
    const progBox = document.getElementById('updater-progress-box');

    currentTagElem.textContent = `v${res.current_version}`;
    latestTagElem.textContent = `v${res.latest_version}`;

    if (!res.has_update) {
      // 1. NO UPDATE NEEDED (Current == Latest)
      statusBadge.textContent = "Up to Date";
      statusBadge.className = "badge-fast";
      sizeCol.style.display = "none";
      notesElem.textContent = `You are currently running the latest version of MTProxy Desk (v${res.current_version}). All core engines and proxies are fully up to date.`;
      progBox.classList.add('hidden');

      startBtn.disabled = true;
      startBtn.className = "btn btn-outline btn-block";
      startBtn.innerHTML = `<span>✔️</span> Running Latest Version (v${res.current_version})`;
      updateActionState = 'idle';

      if (interactive) {
        openUpdaterModal();
      }
    } else {
      // 2. NEW UPDATE AVAILABLE (Latest > Current)
      sizeCol.style.display = "block";
      sizeElem.textContent = res.file_size_mb > 0 ? `${res.file_size_mb} MB` : 'Installer';
      notesElem.textContent = res.release_notes || "Performance optimizations, fresher proxies, and core fixes.";

      if (res.is_downloaded) {
        // Already downloaded on disk
        statusBadge.textContent = "Ready to Install";
        statusBadge.className = "badge-rust";
        progBox.classList.remove('hidden');
        document.getElementById('updater-progress-status').textContent = "✅ Download verified & ready to install!";
        document.getElementById('updater-progress-mb').textContent = `${res.file_size_mb} MB / ${res.file_size_mb} MB (100%)`;
        document.getElementById('updater-progress-bar').style.width = '100%';

        startBtn.disabled = false;
        startBtn.className = "btn btn-success btn-block";
        startBtn.innerHTML = `<span>⚡</span> Install Update Now (تثبيت التحديث الآن)`;
        updateActionState = 'ready_to_install';
      } else {
        // Needs download
        statusBadge.textContent = "Update Available";
        statusBadge.className = "badge-ping ping-fast";
        progBox.classList.add('hidden');

        startBtn.disabled = false;
        startBtn.className = "btn btn-success btn-block";
        startBtn.innerHTML = `<span>📥</span> Download Update (${res.file_size_mb} MB)`;
        updateActionState = 'idle';
      }

      openUpdaterModal();
    }
  } catch (err) {
    console.error("Check update error:", err);
    if (interactive) {
      showToast("Error checking updates: " + err);
    }
  }
}

function openUpdaterModal() {
  document.getElementById('modal-updater').classList.remove('hidden');
}

function closeUpdaterModal() {
  document.getElementById('modal-updater').classList.add('hidden');
}

async function handleUpdaterAction() {
  if (!currentUpdateInfo || !currentUpdateInfo.has_update) {
    return;
  }

  const startBtn = document.getElementById('btn-start-install-update');

  // If already downloaded or in ready_to_install state, trigger installation!
  if (updateActionState === 'ready_to_install' || currentUpdateInfo.is_downloaded) {
    if (!currentUpdateInfo.local_installer_path) {
      showToast("Installer path not found");
      return;
    }
    updateActionState = 'installing';
    startBtn.disabled = true;
    startBtn.innerHTML = `<span>⏳</span> Launching installer & closing app...`;
    showToast("Launching update installer...");

    try {
      await invokeBackend('execute_app_installer', {
        installerPath: currentUpdateInfo.local_installer_path
      });
    } catch (err) {
      showToast("Install error: " + err);
      startBtn.disabled = false;
      startBtn.innerHTML = `<span>⚡</span> Retry Install`;
      updateActionState = 'ready_to_install';
    }
    return;
  }

  // Otherwise, start download!
  updateActionState = 'downloading';
  startBtn.disabled = true;
  startBtn.innerHTML = `<span>⏳</span> Downloading Update (${currentUpdateInfo.file_size_mb} MB)...`;

  const progBox = document.getElementById('updater-progress-box');
  progBox.classList.remove('hidden');
  document.getElementById('updater-progress-status').textContent = "Downloading installer from GitHub...";
  document.getElementById('updater-progress-mb').textContent = `0.0 MB / ${currentUpdateInfo.file_size_mb} MB (0%)`;
  document.getElementById('updater-progress-bar').style.width = '0%';

  try {
    const downloadedPath = await invokeBackend('download_app_update', {
      downloadUrl: currentUpdateInfo.download_url,
      fileName: currentUpdateInfo.file_name || "MTProxy_Desk_Setup_v1.0.2.exe"
    });

    currentUpdateInfo.is_downloaded = true;
    currentUpdateInfo.local_installer_path = downloadedPath;
    updateActionState = 'ready_to_install';

    document.getElementById('updater-progress-status').textContent = "✅ Download complete! Click below to install.";
    document.getElementById('updater-progress-mb').textContent = `${currentUpdateInfo.file_size_mb} MB / ${currentUpdateInfo.file_size_mb} MB (100%)`;
    document.getElementById('updater-progress-bar').style.width = '100%';

    // Transform button to Install button as requested: "المفروض عند انتهاء التنزيل يظهر زر تثبيت"
    startBtn.disabled = false;
    startBtn.className = "btn btn-success btn-block";
    startBtn.innerHTML = `<span>⚡</span> Install Update Now (تثبيت التحديث الآن)`;
    showToast("Download complete! Click 'Install Update Now' to proceed.");
  } catch (err) {
    showToast("Download failed: " + err);
    startBtn.disabled = false;
    startBtn.innerHTML = `<span>🔄</span> Retry Download`;
    updateActionState = 'idle';
  }
}

// Setup
document.addEventListener('DOMContentLoaded', async () => {
  document.querySelectorAll('.nav-btn').forEach(btn => {
    btn.addEventListener('click', () => switchView(btn.dataset.view));
  });

  document.getElementById('btn-lang-toggle').addEventListener('click', () => {
    setLanguage(currentLang === 'en' ? 'ar' : 'en');
  });

  document.getElementById('btn-fetch').addEventListener('click', handleFetchProxies);
  document.getElementById('btn-sidebar-update').addEventListener('click', handleFetchProxies);
  document.getElementById('btn-validate-all').addEventListener('click', handleValidateAll);
  document.getElementById('btn-connect-fastest').addEventListener('click', handleConnectFastest);
  document.getElementById('btn-purge-dead').addEventListener('click', handlePurgeDeadProxies);

  document.getElementById('input-search').addEventListener('input', (e) => {
    searchQuery = e.target.value;
    renderProxiesTable();
  });

  document.querySelectorAll('.chip-group .chip').forEach(chip => {
    chip.addEventListener('click', () => {
      document.querySelectorAll('.chip-group .chip').forEach(c => c.classList.remove('active'));
      chip.classList.add('active');
      currentFilter = chip.dataset.filter;
      renderProxiesTable();
    });
  });

  document.getElementById('select-sort').addEventListener('change', (e) => {
    currentSort = e.target.value;
    renderProxiesTable();
  });

  document.getElementById('btn-copy-tg').addEventListener('click', () => {
    copyToClipboard(document.getElementById('share-tg-link').value, i18n[currentLang].copiedLink);
  });
  document.getElementById('btn-copy-http').addEventListener('click', () => {
    copyToClipboard(document.getElementById('share-http-link').value, i18n[currentLang].copiedLink);
  });
  document.getElementById('btn-studio-connect-tg').addEventListener('click', () => {
    if (activeShareProxy) handleLaunchTelegram(activeShareProxy.id);
  });
  document.getElementById('btn-download-txt').addEventListener('click', async () => {
    if (!activeShareProxy) return;
    try {
      const res = await invokeBackend('export_txt', { id: activeShareProxy.id });
      showToast(res ? `Saved: ${res}` : "Exported successfully");
    } catch (e) {
      showToast("Export error");
    }
  });

  document.getElementById('btn-refresh-logs').addEventListener('click', refreshDiagnosticsLogs);
  document.getElementById('btn-open-log-file').addEventListener('click', () => invokeBackend('open_diagnostics_file'));
  document.getElementById('btn-export-report').addEventListener('click', async () => {
    const res = await invokeBackend('get_diagnostics_report');
    showToast(res ? `Report exported` : "Exported");
  });
  document.getElementById('btn-clear-logs').addEventListener('click', async () => {
    await invokeBackend('clear_diagnostics_logs');
    refreshDiagnosticsLogs();
    showToast("Logs cleared.");
  });

  document.getElementById('btn-save-settings').addEventListener('click', saveAllSettings);
  document.getElementById('btn-add-source').addEventListener('click', async () => {
    const input = document.getElementById('input-new-source');
    const url = input.value.trim();
    if (!url) return;
    try {
      await invokeBackend('add_source', { name: "Custom Source", url });
      input.value = '';
      loadSettingsView();
      showToast("Feed added successfully");
    } catch (e) {
      showToast("Failed to add feed");
    }
  });

  // Software Updater Setup
  document.getElementById('btn-sidebar-check-release').addEventListener('click', () => checkAppUpdates(true));
  document.getElementById('btn-settings-check-update').addEventListener('click', () => checkAppUpdates(true));
  document.getElementById('btn-close-updater').addEventListener('click', closeUpdaterModal);
  document.getElementById('modal-updater').addEventListener('click', (e) => {
    if (e.target.id === 'modal-updater') closeUpdaterModal();
  });
  document.getElementById('btn-start-install-update').addEventListener('click', handleUpdaterAction);
  document.getElementById('btn-open-github-releases').addEventListener('click', () => {
    window.open("https://github.com/moshraheem-sudo/MTProxy-Desk/releases", "_blank");
  });

  // Listen to Tauri progress event
  if (window.__TAURI__ && window.__TAURI__.event && window.__TAURI__.event.listen) {
    window.__TAURI__.event.listen('app-update-progress', (event) => {
      const p = event.payload;
      if (!p) return;
      const progBox = document.getElementById('updater-progress-box');
      progBox.classList.remove('hidden');
      document.getElementById('updater-progress-mb').textContent = `${p.downloaded_mb.toFixed(2)} MB / ${p.total_mb.toFixed(2)} MB (${p.percentage.toFixed(1)}%)`;
      document.getElementById('updater-progress-bar').style.width = `${p.percentage}%`;
      if (p.status === 'completed') {
        document.getElementById('updater-progress-status').textContent = "✅ Download complete! Click below to install.";
        const startBtn = document.getElementById('btn-start-install-update');
        if (startBtn && updateActionState !== 'installing') {
          startBtn.disabled = false;
          startBtn.className = "btn btn-success btn-block";
          startBtn.innerHTML = `<span>⚡</span> Install Update Now (تثبيت التحديث الآن)`;
          updateActionState = 'ready_to_install';
        }
      } else {
        document.getElementById('updater-progress-status').textContent = `Downloading update... ${p.percentage.toFixed(0)}%`;
      }
    });
  }

  // Check for updates silently in background on startup after 3 seconds
  setTimeout(() => {
    checkAppUpdates(false);
  }, 3000);

  // Modal Close
  document.getElementById('btn-close-details').addEventListener('click', closeDetailsModal);
  document.getElementById('modal-details').addEventListener('click', (e) => {
    if (e.target.id === 'modal-details') closeDetailsModal();
  });

  // Dismiss context menu on click elsewhere
  document.addEventListener('click', () => hideContextMenu());
  document.addEventListener('keydown', (e) => {
    if (e.key === 'Escape') {
      hideContextMenu();
      closeDetailsModal();
    }
  });

  // Context Menu Item Actions
  document.getElementById('ctx-connect-tg').addEventListener('click', () => {
    if (activeContextProxy) handleLaunchTelegram(activeContextProxy.id);
    hideContextMenu();
  });
  document.getElementById('ctx-retest').addEventListener('click', () => {
    if (activeContextProxy) handleRetestSingle(activeContextProxy.id);
    hideContextMenu();
  });
  document.getElementById('ctx-toggle-fav').addEventListener('click', () => {
    if (activeContextProxy) handleToggleFavorite(activeContextProxy.id);
    hideContextMenu();
  });
  document.getElementById('ctx-share').addEventListener('click', () => {
    if (activeContextProxy) handleOpenShare(activeContextProxy.id);
    hideContextMenu();
  });
  document.getElementById('ctx-copy-tg').addEventListener('click', () => {
    if (activeContextProxy) {
      const link = `tg://proxy?server=${encodeURIComponent(activeContextProxy.server)}&port=${activeContextProxy.port}&secret=${encodeURIComponent(activeContextProxy.secret)}`;
      copyToClipboard(link, i18n[currentLang].copiedLink);
    }
    hideContextMenu();
  });
  document.getElementById('ctx-copy-http').addEventListener('click', () => {
    if (activeContextProxy) {
      const link = `https://t.me/proxy?server=${encodeURIComponent(activeContextProxy.server)}&port=${activeContextProxy.port}&secret=${encodeURIComponent(activeContextProxy.secret)}`;
      copyToClipboard(link, i18n[currentLang].copiedLink);
    }
    hideContextMenu();
  });
  document.getElementById('ctx-copy-host').addEventListener('click', () => {
    if (activeContextProxy) copyToClipboard(activeContextProxy.server, i18n[currentLang].copiedServer);
    hideContextMenu();
  });
  document.getElementById('ctx-copy-port').addEventListener('click', () => {
    if (activeContextProxy) copyToClipboard(activeContextProxy.port, i18n[currentLang].copiedPort);
    hideContextMenu();
  });
  document.getElementById('ctx-copy-secret').addEventListener('click', () => {
    if (activeContextProxy) copyToClipboard(activeContextProxy.secret, i18n[currentLang].copiedSecret);
    hideContextMenu();
  });
  document.getElementById('ctx-view-details').addEventListener('click', () => {
    if (activeContextProxy) openDetailsModal(activeContextProxy.id);
    hideContextMenu();
  });
  document.getElementById('ctx-delete').addEventListener('click', () => {
    if (activeContextProxy) handleDeleteProxy(activeContextProxy.id);
    hideContextMenu();
  });

  // Load Initial Data
  try {
    let proxies = await invokeBackend('get_proxies');
    if (!proxies || proxies.length === 0) {
      proxies = await invokeBackend('fetch_proxies');
    }
    if (proxies && proxies.length > 0) {
      allProxies = proxies;
    }
  } catch (e) {
    console.error("Initial load error", e);
  }

  await refreshStats();
  renderProxiesTable();
});
