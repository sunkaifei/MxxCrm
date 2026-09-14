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
    var payload = {
      product_id: productId,
      product_name: el ? (el.getAttribute('data-name') || '') : '',
      product_image: '',
      price: parseFloat(el ? (el.getAttribute('data-price') || '0') : '0') || 0,
      quantity: 1,
    };
    // 多规格：附带所选 SKU（库存/价格按规格维度，无全局库存）
    var sel = currentSku(productId);
    if (sel && sel.skuId) {
      payload.sku_id = Number(sel.skuId);
      if (sel.skuCode) { payload.sku_code = sel.skuCode; }
      if (sel.label) { payload.sku_specs = sel.label; }
      if (sel.imageUrl) { payload.product_image = sel.imageUrl; }
      if (sel.price !== undefined && sel.price !== null && sel.price !== '') {
        payload.price = Number(sel.price) || 0;
      }
    }
    return payload;
  }

  /* 当前所选规格（由规格选择模块写入 window.cmsSkuSelection[productId]） */
  function currentSku(productId) {
    return (window.cmsSkuSelection && window.cmsSkuSelection[productId]) || null;
  }

  /* 可购买校验：有规格的产品必须先选到有货规格，否则不可加购/购买 */
  function skuPurchasable(productId) {
    var data = window.cmsSkuData && window.cmsSkuData[productId];
    if (!data || !data.skus || data.skus.length === 0) { return true; }
    var sel = currentSku(productId);
    if (!sel || !sel.skuId) { alert('请选择完整规格'); return false; }
    if (Number(sel.stock) <= 0) { alert('该规格暂时缺货，请选择其他规格'); return false; }
    return true;
  }

  window.cmsAddCart = function (productId) {
    if (!ensureLogin()) { return; }
    if (!skuPurchasable(productId)) { return; }
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
    if (!skuPurchasable(productId)) { return; }
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

/* ============================================================
 * 产品详情规格选择（淘宝式）
 * 数据来自模板内联 <script type="application/json" id="cms-sku-data">：
 *   { specs: [{ name, values: [{ value }] }],
 *     skus:  [{ id, skuCode, label, specs: {规格名:值}, price, stock, imageUrl }] }
 * 行为：无全局库存——库存/价格随所选规格显示；
 *       会导致无货的规格值置灰不可选；缺货规格不可加入购物车/立即购买。
 * 输出：window.cmsSkuData[productId] / window.cmsSkuSelection[productId]（供加购取 sku_id）
 * ============================================================ */
(function () {
  'use strict';

  var dataEl = document.getElementById('cms-sku-data');
  var picker = document.getElementById('cms-spec-picker');
  if (!dataEl || !picker) { return; }

  var payload;
  try { payload = JSON.parse(dataEl.textContent || '{}'); } catch (e) { return; }
  var specs = (payload && payload.specs) || [];
  var skus = (payload && payload.skus) || [];
  if (!specs.length || !skus.length) { return; }

  var productId = picker.getAttribute('data-product-id') || '';
  var buttons = Array.prototype.slice.call(picker.querySelectorAll('.cms-spec-value'));
  var names = specs.map(function (g) { return g.name; });
  var selected = {};

  window.cmsSkuData = window.cmsSkuData || {};
  window.cmsSkuData[productId] = payload;
  window.cmsSkuSelection = window.cmsSkuSelection || {};

  function specOf(sku, name) {
    var sp = sku.specs;
    if (!sp || typeof sp !== 'object') { return ''; }
    var v = sp[name];
    return (v === undefined || v === null) ? '' : String(v);
  }

  // 选完所有规格时定位 SKU；未选完返回 undefined；组合不存在返回 null
  function matchSku() {
    for (var i = 0; i < names.length; i++) {
      if (!selected[names[i]]) { return undefined; }
    }
    return skus.filter(function (s) {
      return names.every(function (n) { return specOf(s, n) === selected[n]; });
    })[0] || null;
  }

  // 在当前已选条件下，选该值是否还能落到有货 SKU
  function valueAvailable(name, value) {
    return skus.some(function (s) {
      if (specOf(s, name) !== value || Number(s.stock) <= 0) { return false; }
      return names.every(function (n) {
        return n === name || !selected[n] || specOf(s, n) === selected[n];
      });
    });
  }

  function formatPrice(v) {
    var n = Number(v) || 0;
    var neg = n < 0;
    n = Math.abs(n);
    var intPart = Math.floor(n);
    var frac = Math.round((n - intPart) * 100);
    if (frac === 100) { frac = 0; intPart += 1; }
    var grouped = String(intPart).replace(/\B(?=(\d{3})+(?!\d))/g, ',');
    return '¥' + (neg ? '-' : '') + grouped + '.' + (frac < 10 ? '0' + frac : String(frac));
  }

  function setButtons(disabled) {
    var box = document.querySelector('.cms-cart-button[data-product-id="' + productId + '"]');
    if (!box) { return; }
    var btns = box.querySelectorAll('.cms-btn-cart, .cms-btn-buy');
    Array.prototype.forEach.call(btns, function (b) {
      b.disabled = disabled;
      b.classList.toggle('disabled', disabled);
    });
  }

  function render() {
    buttons.forEach(function (b) {
      var name = b.getAttribute('data-spec-name');
      var value = b.getAttribute('data-spec-value');
      var isSelected = selected[name] === value;
      b.classList.toggle('active', isSelected);
      var off = !valueAvailable(name, value) && !isSelected;
      b.disabled = off;
      b.classList.toggle('disabled', off);
    });

    var priceEl = document.getElementById('cms-sku-price');
    var stockEl = document.getElementById('cms-sku-stock');
    var sku = matchSku();

    if (sku === undefined) {
      window.cmsSkuSelection[productId] = null;
      if (priceEl) { priceEl.textContent = '价格请选择规格'; }
      if (stockEl) { stockEl.textContent = '请选择规格'; stockEl.className = 'text-muted'; }
      setButtons(true);
      return;
    }
    if (sku === null) {
      window.cmsSkuSelection[productId] = null;
      if (priceEl) { priceEl.textContent = '该规格组合不存在'; }
      if (stockEl) { stockEl.textContent = '该规格组合不存在'; stockEl.className = 'text-danger'; }
      setButtons(true);
      return;
    }

    var stock = Number(sku.stock) || 0;
    if (priceEl && sku.price !== undefined && sku.price !== null && sku.price !== '') {
      priceEl.textContent = formatPrice(sku.price);
    }
    if (stockEl) {
      stockEl.textContent = stock > 0 ? ('库存 ' + stock + ' 件') : '该规格暂时缺货';
      stockEl.className = stock > 0 ? 'text-success' : 'text-danger';
    }
    window.cmsSkuSelection[productId] = {
      skuId: sku.id,
      skuCode: sku.skuCode,
      label: sku.label,
      price: sku.price,
      stock: stock,
      imageUrl: sku.imageUrl,
    };
    var box = document.querySelector('.cms-cart-button[data-product-id="' + productId + '"]');
    if (box && sku.price !== undefined && sku.price !== null && sku.price !== '') {
      box.setAttribute('data-price', String(Number(sku.price) || 0));
    }
    setButtons(stock <= 0);
  }

  buttons.forEach(function (b) {
    b.addEventListener('click', function () {
      var name = b.getAttribute('data-spec-name');
      var value = b.getAttribute('data-spec-value');
      if (selected[name] === value) { delete selected[name]; } else { selected[name] = value; }
      render();
    });
  });

  render();
})();
