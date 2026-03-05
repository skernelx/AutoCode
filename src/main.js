// ============================================================
// AutoCode 前端 — 主页 + 设置面板
// ============================================================

const history = [];
const MAX_HISTORY = 20;
let currentConfig = null;  // 当前配置对象

// ===================== 初始化 =====================
function initApp() {
  setupTabs();
  setupDelaySlider();

  if (window.__TAURI__) {
    // 版本号
    window.__TAURI__.core.invoke('get_version').then(function (v) {
      document.getElementById('version').textContent = 'v' + v;
    });

    // 加载配置
    window.__TAURI__.core.invoke('get_config').then(function (cfg) {
      currentConfig = cfg;
      applyConfigToUI(cfg);
    });

    // 监听验证码事件
    window.__TAURI__.event.listen('verification-code', function (ev) {
      var d = ev.payload;
      showCode(d.code, d.source, d.strategy, d.confidence);
    });

    // 监听后端发起的跳转到设置页
    window.__TAURI__.event.listen('navigate', function (ev) {
      if (ev.payload === 'settings') {
        switchTab('settings');
      }
    });

    // 启动时检查权限
    checkPermissions();

    // 启动时检查更新
    checkForUpdates();
  }

  // 按钮事件
  document.getElementById('btn-save').addEventListener('click', saveConfig);
  document.getElementById('btn-reset').addEventListener('click', resetConfig);
  document.getElementById('copy-btn').addEventListener('click', copyLatestCode);

  // 权限检测按钮
  document.getElementById('btn-open-fda').addEventListener('click', function () {
    if (window.__TAURI__) window.__TAURI__.core.invoke('open_fda_settings');
  });
  document.getElementById('btn-open-acc').addEventListener('click', function () {
    if (window.__TAURI__) window.__TAURI__.core.invoke('open_accessibility_settings');
  });
  document.getElementById('btn-recheck').addEventListener('click', checkPermissions);

  // 添加项目按钮
  document.getElementById('add-keyword').addEventListener('click', function () {
    showInlineInput('keyword-list', function (val) {
      if (currentConfig) currentConfig.verification_keywords.push(val);
      renderKeywords();
    });
  });
  document.getElementById('add-sender').addEventListener('click', function () {
    showInlineInput('sender-list', function (val) {
      if (currentConfig) currentConfig.known_2fa_senders.push(val);
      renderSenders();
    });
  });
  document.getElementById('add-pattern').addEventListener('click', function () {
    showInlineInput('pattern-list', function (val) {
      if (currentConfig) currentConfig.verification_patterns.push(val);
      renderPatterns();
    });
  });
  document.getElementById('add-autofill-app').addEventListener('click', function () {
    showInlineInput('autofill-app-list', function (val) {
      if (currentConfig) currentConfig.native_autofill_apps.push(val);
      renderAutofillApps();
    });
  });
}

// ===================== 权限检测 =====================
function checkPermissions() {
  if (!window.__TAURI__) return;

  window.__TAURI__.core.invoke('check_permissions').then(function (status) {
    var banner = document.getElementById('perm-banner');
    var fdaEl = document.getElementById('perm-fda');
    var accEl = document.getElementById('perm-acc');
    var titleEl = banner.querySelector('.perm-title');

    var needsFDA = !status.full_disk_access;
    var needsAcc = !status.accessibility;

    fdaEl.style.display = needsFDA ? 'flex' : 'none';
    accEl.style.display = needsAcc ? 'flex' : 'none';

    if (needsFDA || needsAcc) {
      banner.style.display = 'block';
      banner.classList.remove('all-granted');
      titleEl.innerHTML =
        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z"/><line x1="12" y1="9" x2="12" y2="13"/><line x1="12" y1="17" x2="12.01" y2="17"/></svg> 需要权限';
    } else {
      // 全部已授权 — 短暂显示成功提示后隐藏
      banner.style.display = 'block';
      banner.classList.add('all-granted');
      titleEl.innerHTML =
        '<svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg> 权限已就绪';
      document.getElementById('btn-recheck').style.display = 'none';
      setTimeout(function () {
        banner.style.display = 'none';
      }, 3000);
    }
  });
}

// ===================== Tab 切换 =====================
function setupTabs() {
  var btns = document.querySelectorAll('.tab-btn');
  btns.forEach(function (btn) {
    btn.addEventListener('click', function () {
      switchTab(btn.getAttribute('data-tab'));
    });
  });
}

function switchTab(name) {
  document.querySelectorAll('.tab-btn').forEach(function (b) {
    b.classList.toggle('active', b.getAttribute('data-tab') === name);
  });
  document.querySelectorAll('.page').forEach(function (p) {
    p.classList.toggle('active', p.id === 'page-' + name);
  });
}

// ===================== 配置 → UI =====================
function applyConfigToUI(cfg) {
  // 主页状态
  setStatus('status-imessage', cfg.listen_imessage);
  setStatus('status-applemail', cfg.listen_apple_mail);
  setStatus('status-outlook', cfg.listen_outlook);

  // 设置开关
  document.getElementById('cfg-listen-imessage').checked = cfg.listen_imessage;
  document.getElementById('cfg-listen-apple-mail').checked = cfg.listen_apple_mail;
  document.getElementById('cfg-listen-outlook').checked = cfg.listen_outlook;
  document.getElementById('cfg-auto-enter').checked = cfg.auto_enter;
  document.getElementById('cfg-launch-at-login').checked = cfg.launch_at_login;

  // 粘贴模式
  var radios = document.querySelectorAll('input[name="paste_mode"]');
  radios.forEach(function (r) {
    r.checked = r.value === cfg.paste_mode;
  });

  // 延迟滑块
  var slider = document.getElementById('cfg-autofill-delay');
  slider.value = cfg.autofill_detect_delay_ms;
  document.getElementById('delay-value').textContent = cfg.autofill_detect_delay_ms + 'ms';

  // 列表
  renderKeywords();
  renderSenders();
  renderPatterns();
  renderAutofillApps();
}

function setStatus(id, active) {
  var el = document.getElementById(id);
  if (el) el.classList.toggle('active', active);
}

// ===================== UI → 配置 =====================
function collectConfigFromUI() {
  if (!currentConfig) return null;

  currentConfig.listen_imessage = document.getElementById('cfg-listen-imessage').checked;
  currentConfig.listen_apple_mail = document.getElementById('cfg-listen-apple-mail').checked;
  currentConfig.listen_outlook = document.getElementById('cfg-listen-outlook').checked;
  currentConfig.auto_enter = document.getElementById('cfg-auto-enter').checked;
  currentConfig.launch_at_login = document.getElementById('cfg-launch-at-login').checked;
  currentConfig.autofill_detect_delay_ms = parseInt(document.getElementById('cfg-autofill-delay').value, 10);

  var checked = document.querySelector('input[name="paste_mode"]:checked');
  if (checked) currentConfig.paste_mode = checked.value;

  return currentConfig;
}

// ===================== 保存 / 重置 =====================
function saveConfig() {
  var cfg = collectConfigFromUI();
  if (!cfg || !window.__TAURI__) return;

  window.__TAURI__.core.invoke('update_config', { newConfig: cfg }).then(function () {
    // 更新主页状态
    setStatus('status-imessage', cfg.listen_imessage);
    setStatus('status-applemail', cfg.listen_apple_mail);
    setStatus('status-outlook', cfg.listen_outlook);
    showToast();
  }).catch(function (err) {
    console.error('保存配置失败:', err);
  });
}

function resetConfig() {
  if (!window.__TAURI__) return;
  // 先保存一个空配置让后端产生默认值，再重新获取
  // 简单方式：直接请求默认值
  if (!confirm('确定要恢复所有设置为默认值吗？')) return;

  // 使用内建的默认值
  window.__TAURI__.core.invoke('get_default_config').then(function (cfg) {
    currentConfig = cfg;
    applyConfigToUI(cfg);
    saveConfig();
  }).catch(function () {
    // 如果后端没有 get_default_config 命令，提示用户
    console.warn('get_default_config 命令不可用');
  });
}

function showToast() {
  var toast = document.getElementById('save-toast');
  toast.classList.add('show');
  setTimeout(function () {
    toast.classList.remove('show');
  }, 2000);
}

// ===================== 验证码展示 =====================
function showCode(code, source, strategy, confidence) {
  var display = document.getElementById('code-display');
  display.style.display = 'block';
  document.getElementById('code-value').textContent = code;
  document.getElementById('code-source').textContent =
    source + ' · ' + strategy + ' · ' + Math.round(confidence * 100) + '%';

  var now = new Date();
  var timeStr = pad(now.getHours()) + ':' + pad(now.getMinutes()) + ':' + pad(now.getSeconds());

  history.unshift({ code: code, source: source, time: timeStr });
  if (history.length > MAX_HISTORY) history.pop();
  renderHistory();
}

function copyLatestCode() {
  if (history.length === 0) return;

  // 检查 clipboard API 是否可用
  if (!navigator.clipboard || !navigator.clipboard.writeText) {
    // 降级方案：使用传统方法
    fallbackCopyToClipboard(history[0].code);
    return;
  }

  navigator.clipboard.writeText(history[0].code).then(function () {
    var btn = document.getElementById('copy-btn');
    btn.classList.add('copied');
    btn.innerHTML =
      '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><polyline points="20 6 9 17 4 12"/></svg> 已复制';
    setTimeout(function () {
      btn.classList.remove('copied');
      btn.innerHTML =
        '<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2"><rect x="9" y="9" width="13" height="13" rx="2" ry="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/></svg> 复制';
    }, 1500);
  }).catch(function (err) {
    console.error('复制失败:', err);
    fallbackCopyToClipboard(history[0].code);
  });
}

// 降级复制方案
function fallbackCopyToClipboard(text) {
  var textArea = document.createElement('textarea');
  textArea.value = text;
  textArea.style.position = 'fixed';
  textArea.style.left = '-999999px';
  document.body.appendChild(textArea);
  textArea.select();
  try {
    document.execCommand('copy');
    var btn = document.getElementById('copy-btn');
    btn.classList.add('copied');
    btn.textContent = '已复制';
    setTimeout(function () {
      btn.classList.remove('copied');
      btn.textContent = '复制';
    }, 1500);
  } catch (err) {
    console.error('降级复制也失败:', err);
  }
  document.body.removeChild(textArea);
}

function renderHistory() {
  var el = document.getElementById('history-list');
  if (history.length === 0) {
    el.innerHTML = '<p class="empty-hint">等待验证码…</p>';
    return;
  }
  var html = '';
  for (var i = 0; i < history.length; i++) {
    var item = history[i];
    html += '<div class="history-item" data-code="' + esc(item.code) + '">' +
            '<span class="code">' + esc(item.code) + '</span>' +
            '<span class="meta">' + esc(item.source) + '<br>' + item.time + '</span>' +
            '</div>';
  }
  el.innerHTML = html;

  // 点击复制
  el.querySelectorAll('.history-item').forEach(function (item) {
    item.addEventListener('click', function () {
      var code = item.getAttribute('data-code');
      if (navigator.clipboard && navigator.clipboard.writeText) {
        navigator.clipboard.writeText(code).catch(function (err) {
          console.error('复制失败:', err);
          fallbackCopyToClipboard(code);
        });
      } else {
        fallbackCopyToClipboard(code);
      }
    });
  });
}

// ===================== 列表渲染 =====================
function renderKeywords() {
  renderTagList('keyword-list', currentConfig ? currentConfig.verification_keywords : [], function (idx) {
    currentConfig.verification_keywords.splice(idx, 1);
    renderKeywords();
  });
}

function renderSenders() {
  renderTagList('sender-list', currentConfig ? currentConfig.known_2fa_senders : [], function (idx) {
    currentConfig.known_2fa_senders.splice(idx, 1);
    renderSenders();
  });
}

function renderAutofillApps() {
  renderTagList('autofill-app-list', currentConfig ? currentConfig.native_autofill_apps : [], function (idx) {
    currentConfig.native_autofill_apps.splice(idx, 1);
    renderAutofillApps();
  });
}

function renderTagList(containerId, items, onRemove) {
  var container = document.getElementById(containerId);
  // 保留可能已存在的 inline-input
  var existingInput = container.querySelector('.inline-input');
  var html = '';
  for (var i = 0; i < items.length; i++) {
    html += '<span class="tag">' +
            esc(items[i]) +
            '<span class="remove" data-idx="' + i + '">&times;</span>' +
            '</span>';
  }
  container.innerHTML = html;
  if (existingInput) container.appendChild(existingInput);

  container.querySelectorAll('.remove').forEach(function (btn) {
    btn.addEventListener('click', function (e) {
      e.stopPropagation();
      onRemove(parseInt(btn.getAttribute('data-idx'), 10));
    });
  });
}

function renderPatterns() {
  var container = document.getElementById('pattern-list');
  var items = currentConfig ? currentConfig.verification_patterns : [];
  var existingInput = container.querySelector('.inline-input');
  var html = '';
  for (var i = 0; i < items.length; i++) {
    html += '<div class="pattern-item">' +
            '<span class="pattern-text">' + esc(items[i]) + '</span>' +
            '<span class="remove" data-idx="' + i + '">&times;</span>' +
            '</div>';
  }
  container.innerHTML = html;
  if (existingInput) container.appendChild(existingInput);

  container.querySelectorAll('.remove').forEach(function (btn) {
    btn.addEventListener('click', function (e) {
      e.stopPropagation();
      var idx = parseInt(btn.getAttribute('data-idx'), 10);
      currentConfig.verification_patterns.splice(idx, 1);
      renderPatterns();
    });
  });
}

// ===================== Inline Input =====================
function showInlineInput(containerId, onAdd) {
  var container = document.getElementById(containerId);
  // 避免重复
  if (container.querySelector('.inline-input')) return;

  var wrapper = document.createElement('div');
  wrapper.className = 'inline-input';
  var input = document.createElement('input');
  input.type = 'text';
  input.placeholder = '输入内容后按回车…';
  var addBtn = document.createElement('button');
  addBtn.textContent = '添加';

  wrapper.appendChild(input);
  wrapper.appendChild(addBtn);
  container.appendChild(wrapper);

  input.focus();

  function commit() {
    var val = input.value.trim();
    if (val) {
      onAdd(val);
      wrapper.remove();
    }
  }

  addBtn.addEventListener('click', commit);
  input.addEventListener('keydown', function (e) {
    if (e.key === 'Enter') commit();
    if (e.key === 'Escape') wrapper.remove();
  });
  input.addEventListener('blur', function () {
    setTimeout(function () { wrapper.remove(); }, 200);
  });
}

// ===================== 延迟滑块 =====================
function setupDelaySlider() {
  var slider = document.getElementById('cfg-autofill-delay');
  var label = document.getElementById('delay-value');
  slider.addEventListener('input', function () {
    label.textContent = slider.value + 'ms';
  });
}

// ===================== 自动更新 =====================
async function checkForUpdates() {
  if (!window.__TAURI__) return;

  try {
    const { check } = window.__TAURI__.updater;
    const { ask } = window.__TAURI__.dialog;
    const { relaunch } = window.__TAURI__.process;

    const update = await check();

    if (update?.available) {
      const yes = await ask(
        `发现新版本 ${update.version}！\n\n当前版本：${update.currentVersion}\n\n是否立即下载并安装？`,
        {
          title: 'AutoCode 更新',
          kind: 'info',
          okLabel: '立即更新',
          cancelLabel: '稍后提醒'
        }
      );

      if (yes) {
        await update.downloadAndInstall();
        await relaunch();
      }
    }
  } catch (error) {
    console.log('检查更新失败:', error);
  }
}

// ===================== 工具函数 =====================
function pad(n) {
  return n < 10 ? '0' + n : '' + n;
}

function esc(text) {
  var d = document.createElement('div');
  d.appendChild(document.createTextNode(text));
  return d.innerHTML;
}

// ===================== 启动 =====================
if (document.readyState === 'loading') {
  document.addEventListener('DOMContentLoaded', initApp);
} else {
  initApp();
}
