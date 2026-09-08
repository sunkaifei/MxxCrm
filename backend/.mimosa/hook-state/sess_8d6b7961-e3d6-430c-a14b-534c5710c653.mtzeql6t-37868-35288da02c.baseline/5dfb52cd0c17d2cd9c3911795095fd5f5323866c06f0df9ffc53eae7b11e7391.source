/* ============================================================
 * MxxCRM 单站 CMS 前台公共脚本（产品模块 P-0/P-1.5，方案 v3.0 §6.4/§6.5）
 * 引入方式：<script src="/static/default/js/cms.js"></script>
 *
 * 提供：cmsOpenLead/cmsCloseLead/cmsSubmitLead（咨询弹窗与提交）
 *       cmsAddCart/cmsBuyNow（交易按钮，登录态存 localStorage.cmsUserToken）
 *       cmsCartBadgeRefresh（页头购物车角标）
 * ============================================================ */

(function () {
  'use strict';

  var LEAD_SUBMIT_URL = '/api/open/leave_msg/submit';
  var CART_ADD_URL = '/api/user/cart/add';
  var CART_LIST_URL = '/api/user/cart/list';
  var LOGIN_PATH = '/user/login';

  function getToken() {
    try { return localStorage.getItem('cmsUserToken') || ''; } catch (e) { return ''; }
  }
  function setToken(t) {
    try {
      if (t) { localStorage.setItem('cmsUserToken', t); } else { localStorage.removeItem('cmsUserToken'); }
    } catch (e) { /* ignore */ }
  }

  function postJSON(url, data, withAuth) {
    var headers = { 'Content-Type': 'application/json' };
    if (withAuth) {
      var t = getToken();
      if (!t) { return Promise.reject({ code: 401, msg: '请先登录' }); }
      headers['Authorization'] = 'Bearer ' + t;
    }
    return fetch(url, { method: 'POST', headers: headers, body: JSON.stringify(data) })
      .then(function (res) { return res.json(); });
  }

  /* ---------- 咨询留言弹窗（lead_form 输出的 DOM 结构） ---------- */

  window.cmsOpenLead = function (productId) {
    var el = document.getElementById('lead-form-' + productId);
    if (!el) { return; }
    el.style.display = 'flex';
    document.body.style.overflow = 'hidden';
    var first = el.querySelector('input[name="contact_name"]');
    if (first) { setTimeout(function () { first.focus(); }, 50); }
  };

  window.cmsCloseLead = function (productId) {
    var el = document.getElementById('lead-form-' + productId);
    if (!el) { return; }
    el.style.display = 'none';
    document.body.style.overflow = '';
  };

  // ESC 关闭 + 点击遮罩关闭
  document.addEventListener('keydown', function (e) {
    if (e.key !== 'Escape') { return; }
    document.querySelectorAll('.cms-lead-modal').forEach(function (m) {
      if (m.style.display !== 'none') {
        m.style.display = 'none';
        document.body.style.overflow = '';
      }
    });
  });
  document.addEventListener('click', function (e) {
    if (e.target && e.target.classList && e.target.classList.contains('cms-lead-modal')) {
      e.target.style.display = 'none';
      document.body.style.overflow = '';
    }
  });

  // 表单提交：fetch + 防重复提交 + 反馈
  window.cmsSubmitLead = function (event, productId) {
    event.preventDefault();
    var form = event.target;
    var btn = form.querySelector('button[type="submit"]');
    var feedback = form.parentElement.querySelector('.cms-lead-feedback');
    var get = function (name) {
      var f = form.querySelector('[name="' + name + '"]');
      return f ? f.value.trim() : '';
    };
    var payload = {
      contact_name: get('contactName'),
      contact_phone: get('contactPhone'),
      contact_email: get('contactEmail'),
      content: get('content'),
      product_id: productId > 0 ? productId : undefined,
      source: 'website',
    };
    if (btn) { btn.disabled = true; }
    postJSON(LEAD_SUBMIT_URL, payload, false)
      .then(function (res) {
        var ok = res && (res.code === 200 || res.code === 0);
        if (feedback) {
          feedback.style.display = 'block';
          feedback.className = 'cms-lead-feedback ' + (ok ? 'ok' : 'err');
          feedback.textContent = ok ? '提交成功，我们会尽快与您联系！' : ('提交失败：' + (res.msg || '请稍后重试'));
        }
        if (ok) {
          form.reset();
          setTimeout(function () { window.cmsCloseLead(productId); }, 1200);
        }
      })
      .catch(function () {
        if (feedback) {
          feedback.style.display = 'block';
          feedback.className = 'cms-lead-feedback err';
          feedback.textContent = '网络异常，请稍后重试';
        }
      })
      .finally(function () {
        if (btn) { btn.disabled = false; }
      });
    return false;
  };

  /* ---------- 购物车（交易型 site_mode=2/3） ---------- */

  function ensureLogin() {
    if (getToken()) { return true; }
    window.location.href = LOGIN_PATH + '?redirect=' + encodeURIComponent(window.location.pathname);
    return false;
  }

  function findButton(el) {
    var node = el;
    while (node && node !== document.body) {
      if (node.classList && node.classList.contains('cms-cart-button')) { return node; }
      node = node.parentElement;
    }
    return null;
  }

  function productPayload(el, productId) {
    return {
      product_id: productId,
      product_name: el ? (el.getAttribute('data-name') || '') : '',
      product_image: '',
      price: parseFloat(el ? (el.getAttribute('data-price') || '0') : '0') || 0,
      quantity: 1,
    };
  }

  window.cmsAddCart = function (productId) {
    if (!ensureLogin()) { return; }
    var el = findButton(document.querySelector('.cms-cart-button[data-product-id="' + productId + '"]'));
    postJSON(CART_ADD_URL, productPayload(el, productId), true)
      .then(function (res) {
        if (res && (res.code === 200 || res.code === 0)) {
          if (el) {
            var btn = el.querySelector('.cms-btn-cart');
            if (btn) {
              btn.classList.add('cms-added');
              btn.textContent = '已加入';
              setTimeout(function () { btn.classList.remove('cms-added'); btn.textContent = '加入购物车'; }, 1200);
            }
          }
          window.cmsCartBadgeRefresh();
        } else if (res && res.code === 400 && String(res.msg || '').indexOf('登录') >= 0) {
          window.location.href = LOGIN_PATH;
        } else {
          alert((res && res.msg) || '加入购物车失败');
        }
      })
      .catch(function () { alert('网络异常，请稍后重试'); });
  };

  window.cmsBuyNow = function (productId) {
    if (!ensureLogin()) { return; }
    var el = findButton(document.querySelector('.cms-cart-button[data-product-id="' + productId + '"]'));
    postJSON(CART_ADD_URL, productPayload(el, productId), true)
      .then(function (res) {
        if (res && (res.code === 200 || res.code === 0)) {
          window.location.href = '/cart';
        } else {
          window.location.href = LOGIN_PATH;
        }
      })
      .catch(function () { alert('网络异常，请稍后重试'); });
  };

  // 页头购物车角标：登录态下拉取数量，写入 .cms-cart-badge
  window.cmsCartBadgeRefresh = function () {
    var badge = document.querySelector('.cms-cart-badge');
    if (!badge || !getToken()) { return; }
    fetch(CART_LIST_URL, { headers: { 'Authorization': 'Bearer ' + getToken() } })
      .then(function (r) { return r.json(); })
      .then(function (res) {
        var count = 0;
        var list = (res && res.data && (res.data.items || res.data)) || [];
        if (Array.isArray(list)) {
          list.forEach(function (it) { count += (it.quantity || 0); });
        }
        badge.textContent = count > 99 ? '99+' : String(count);
        badge.classList.toggle('show', count > 0);
      })
      .catch(function () { /* 静默 */ });
  };

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', window.cmsCartBadgeRefresh);
  } else {
    window.cmsCartBadgeRefresh();
  }

  // 登录页表单（/user/login SSR 页由 cms.js 处理提交，token 存 localStorage）
  window.cmsLogin = function (event) {
    event.preventDefault();
    var form = event.target;
    var get = function (name) {
      var f = form.querySelector('[name="' + name + '"]');
      return f ? f.value.trim() : '';
    };
    postJSON('/api/open/website/user/login', { account: get('account'), password: get('password') }, false)
      .then(function (res) {
        if (res && (res.code === 200 || res.code === 0)) {
          var vo = res.data || {};
          setToken(vo.token || vo.accessToken || '');
          var params = new URLSearchParams(window.location.search);
          window.location.href = params.get('redirect') || '/cart';
        } else {
          var tip = document.getElementById('cms-login-tip');
          if (tip) { tip.textContent = (res && res.msg) || '登录失败'; tip.style.display = 'block'; }
        }
      })
      .catch(function () { alert('网络异常，请稍后重试'); });
    return false;
  };
})();

/* ============================================================
 * 导航可见性过滤（T-P2.1）
 * 依据页头/页脚动态导航 <a> 上的 data-nav-guest / data-nav-devices 属性，
 * 在客户端按（是否登录 × 当前设备）隐藏不可见项：
 *   data-nav-guest="0"           → 仅登录用户可见（无 cmsUserToken 时隐藏）
 *   data-nav-devices="pc,mobile" → 仅列出的设备可见（空=全部）
 * 说明：CmsTagData 预取无请求上下文，故为渲染侧过滤；不涉及敏感数据。
 * ============================================================ */
(function () {
  'use strict';

  function currentDevice() {
    var ua = navigator.userAgent || '';
    return /Mobi|Android|iPhone|iPad|iPod|Windows Phone/i.test(ua) ? 'mobile' : 'pc';
  }

  function cmsNavVisibility() {
    var isLogin = false;
    try { isLogin = !!localStorage.getItem('cmsUserToken'); } catch (e) { isLogin = false; }
    var device = currentDevice();
    var anchors = document.querySelectorAll('a[data-nav-guest]');
    Array.prototype.forEach.call(anchors, function (a) {
      var guest = a.getAttribute('data-nav-guest');
      var devices = (a.getAttribute('data-nav-devices') || '').trim();
      var hide = false;
      if (guest === '0' && !isLogin) { hide = true; }
      if (!hide && devices) {
        var list = devices.split(',').map(function (s) { return s.trim(); });
        if (list.indexOf(device) < 0) { hide = true; }
      }
      if (hide) { a.style.display = 'none'; }
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', cmsNavVisibility);
  } else {
    cmsNavVisibility();
  }
})();

/* ============================================================
 * 导航 active 高亮（T-P2.3）
 * 依据当前 window.location.pathname 与导航链接比对，为命中的链接追加
 * .active 类并写入 aria-current="page"，实现当前页导航高亮。
 * 覆盖：页头动态导航（header nav a）、企业版下拉/移动导航（.fxk-links a）、
 *       页脚导航（.cms-footer-nav a）。
 * 规则：精确匹配优先；对非根路径追加「当前路径以链接路径 + '/' 开头」的子级匹配。
 * ============================================================ */
(function () {
  'use strict';

  function normPath(p) {
    if (!p) { return '/'; }
    if (p.length > 1 && p.charAt(p.length - 1) === '/') { p = p.slice(0, -1); }
    return p;
  }

  function cmsNavHighlight() {
    var current = normPath(window.location.pathname || '/');
    var anchors = document.querySelectorAll(
      'header nav a, .fxk-links a, .cms-footer-nav a, .cms-nav a'
    );
    Array.prototype.forEach.call(anchors, function (a) {
      var href = a.getAttribute('href') || '';
      if (!href || href.charAt(0) === '#' || href.indexOf('javascript:') === 0) { return; }
      var url;
      try { url = new URL(a.href, window.location.origin); } catch (e) { return; }
      if (url.origin !== window.location.origin) { return; }
      var linkPath = normPath(url.pathname);
      var matched =
        linkPath === current ||
        (linkPath !== '/' && current.indexOf(linkPath + '/') === 0);
      if (matched) {
        a.classList.add('active');
        a.setAttribute('aria-current', 'page');
      }
    });
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', cmsNavHighlight);
  } else {
    cmsNavHighlight();
  }
})();
