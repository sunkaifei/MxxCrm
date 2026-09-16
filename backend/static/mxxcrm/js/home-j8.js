
  (function ($) {
    if (window.__semRegFormBootstrapped) {
      return;
    }
    window.__semRegFormBootstrapped = true;
    var REG_FORM_IDS = {
      test: "30cdc96d95624ccc8416441a318667e4",
      prodPc: "5ae44797fc09420498993cd58159135a",
      prodMobile: "251ce5a9f1a34edebf0f12532011cb8e",
    };
    var REG_QRCODE_CONFIG = {
      type: "qw",
      testId: "f782662c303f440dbd6d4e58e18e7636",
      prodId: "0875ee2ad8384ed0be7b913a0672fd89",
    };
    var REG_QRCODE_FALLBACK_URL =
      "https://a9.fspage.com/FSR/official-site/img/web-reg/qywx-default-qrcode.png";

    // 第二步补充提交的字段 apiName（暂写死，后续可从后台配置扩展）
    var STEP2_FIELD_API_NAMES = ["texts1_60ab8fbb4e6cbeb9", "texts1_4ad99b683625440c"];
    var OTHER_OPTION_VALUE = "other";

    var state = {
      formId: "",
      enrollId: "",
      formFields: [],
      step2Field: null,
      codeTimer: null,
      codeCountdown: 0,
      selectedOptions: {},
      otherText: "",
      step1Handled: false,
      needVerifyCode: false,
      qrcodeUrl: "",
      qrcodeLoading: false,
    };

    function getFieldByApiName(apiName) {
      return state.formFields.find(function (item) {
        return item.apiName === apiName;
      });
    }

    function needPhoneVerifyCode(fields) {
      if (!fields || !fields.length) {
        return false;
      }
      var verifyField = fields.find(function (item) {
        return item.apiName === "phoneVerifyCode";
      });
      if (verifyField) {
        return true;
      }
      var phoneField = fields.find(function (item) {
        return item.apiName === "phone";
      });
      return !!(phoneField && phoneField.isVerify);
    }

    function getFieldHelpText(field) {
      return field && field.helpText ? field.helpText : "";
    }

    function getFormHintsCacheKey() {
      return "reg_form_hints_" + state.formId;
    }

    function getVerifyCodePlaceholder(verifyField) {
      var helpText = getFieldHelpText(verifyField);
      if (helpText) {
        return helpText;
      }
      return state.needVerifyCode ? "请输入验证码" : "";
    }

    function applyCachedFormFieldHints() {
      try {
        var cached = JSON.parse(localStorage.getItem(getFormHintsCacheKey()) || "null");
        if (!cached) {
          return;
        }
        if (cached.phonePlaceholder) {
          $(".js-reg-phone").attr("placeholder", cached.phonePlaceholder);
        }
        if (cached.companyPlaceholder) {
          $(".js-reg-company").attr("placeholder", cached.companyPlaceholder);
        }
        if (cached.needVerifyCode) {
          state.needVerifyCode = true;
          $(".js-reg-code-field").addClass("is-visible");
          $(".js-reg-code").attr(
            "placeholder",
            cached.codePlaceholder || "请输入验证码"
          );
        }
      } catch (err) {
        console.warn("读取表单缓存失败", err);
      }
    }

    function saveFormFieldHintsCache(codePlaceholder) {
      try {
        localStorage.setItem(
          getFormHintsCacheKey(),
          JSON.stringify({
            needVerifyCode: state.needVerifyCode,
            phonePlaceholder: getFieldHelpText(getFieldByApiName("phone")),
            companyPlaceholder: getFieldHelpText(getFieldByApiName("companyName")),
            codePlaceholder: codePlaceholder,
          })
        );
      } catch (err) {
        console.warn("写入表单缓存失败", err);
      }
    }

    function applyFormFieldHints() {
      state.needVerifyCode = needPhoneVerifyCode(state.formFields);
      if (state.needVerifyCode) {
        $(".js-reg-code-field").addClass("is-visible");
      } else {
        $(".js-reg-code-field").removeClass("is-visible");
      }

      $(".js-reg-phone").attr("placeholder", getFieldHelpText(getFieldByApiName("phone")));
      $(".js-reg-company").attr(
        "placeholder",
        getFieldHelpText(getFieldByApiName("companyName"))
      );

      var verifyField = getFieldByApiName("phoneVerifyCode");
      var codePlaceholder = getVerifyCodePlaceholder(verifyField);
      $(".js-reg-code").attr("placeholder", codePlaceholder);

      state.step2Field = findStep2Field(state.formFields);
      var $step2Label = $(".js-reg-step2-label");
      if (state.step2Field) {
        var helpText = getFieldHelpText(state.step2Field);
        var labelHtml = state.step2Field.isRequired ? "<em>*</em>" : "";
        labelHtml += helpText;
        $step2Label.html(labelHtml);
      } else {
        $step2Label.empty();
      }

      saveFormFieldHintsCache(codePlaceholder);
      $(".js-reg-form-fields").removeClass("is-fields-pending");
      if (isDesktopForm()) {
        preloadRegSuccessQrcode();
      }
    }

    function clearFieldErrors() {
      $(".js-reg-field").removeClass("is-empty-error is-value-error");
      $(".js-reg-field .reg-field__error").removeClass("is-visible").text("");
    }

    function clearStep2Errors() {
      $(".js-reg-step2-field").removeClass("is-value-error");
      $(".js-reg-step2-error").removeClass("is-visible").text("");
    }

    function setEmptyFieldError(fieldName) {
      $('.js-reg-field[data-field="' + fieldName + '"]').addClass("is-empty-error");
    }

    function setValueFieldError(fieldName, message) {
      var $field = $('.js-reg-field[data-field="' + fieldName + '"]');
      $field.addClass("is-value-error");
      $field.find(".reg-field__error").text(message).addClass("is-visible");
    }

    function setStep2ValueError(message) {
      $(".js-reg-step2-field").addClass("is-value-error");
      $(".js-reg-step2-error").text(message).addClass("is-visible");
    }

    function clearFieldErrorByEl($input) {
      var $field = $input.closest(".js-reg-field, .js-reg-step2-field");
      $field.removeClass("is-empty-error is-value-error");
      $field.find(".reg-field__error").removeClass("is-visible").text("");
    }

    function isTestEnv() {
      return window.origin.indexOf("ceshi112") !== -1;
    }

    function isDesktopForm() {
      return window.innerWidth > 800;
    }

    function getFormId() {
      if (isTestEnv()) {
        return REG_FORM_IDS.test;
      }
      return REG_FORM_IDS.prodPc;
    }

    function getRegQrcodeId() {
      return isTestEnv() ? REG_QRCODE_CONFIG.testId : REG_QRCODE_CONFIG.prodId;
    }

    function waitForFormSDK() {
      return new Promise(function (resolve, reject) {
        if (window.FsYxt && window.FsYxt.FormSDK) {
          resolve();
          return;
        }

        var attempts = 0;
        var timer = null;
        var sdkLoadBound = false;

        function cleanup() {
          if (timer) {
            clearInterval(timer);
            timer = null;
          }
        }

        function tryResolve() {
          if (window.FsYxt && window.FsYxt.FormSDK) {
            cleanup();
            resolve();
            return true;
          }
          return false;
        }

        function bindSdkLoadListener() {
          if (sdkLoadBound) {
            return;
          }
          var sdkScript = document.getElementById("fsMarketingWebsiteScript");
          if (!sdkScript) {
            return;
          }
          sdkLoadBound = true;
          sdkScript.addEventListener("load", function () {
            tryResolve();
          });
        }

        timer = setInterval(function () {
          bindSdkLoadListener();
          attempts += 1;
          if (tryResolve()) {
            return;
          }
          if (attempts >= 200) {
            cleanup();
            reject(new Error("FormSDK not ready"));
          }
        }, 50);
      });
    }

    function uet_report_conversion() {
      window.uetq = window.uetq || [];
      window.uetq.push("event", "demo", {});
    }

    function readCookie(name) {
      if (typeof $.cookie === "function") {
        return $.cookie(name);
      }
      var escaped = name.replace(/[.$?*|{}()[\]\\/+^]/g, "\\$&");
      var match = document.cookie.match(new RegExp("(?:^|; )" + escaped + "=([^;]*)"));
      return match ? decodeURIComponent(match[1]) : "";
    }

    function writeCookie(name, value) {
      if (typeof $.cookie === "function") {
        $.cookie(name, value, { path: "/" });
        return;
      }
      document.cookie =
        name + "=" + encodeURIComponent(value) + ";path=/";
    }

    function fireStep1Tracking() {
      try {
        uet_report_conversion();
        window._hmt && window._hmt.push(["_trackCustomEvent", "try_success", {}]);
        window._agl && window._agl.push(["track", ["success", { t: 18 }]]);
        window._agl && window._agl.push(["track", ["success", { t: 3 }]]);
        var partnerID = readCookie("fxuuid");
        if (partnerID) {
          writeCookie("partnerID", partnerID);
        }
        writeCookie("infostatus", "已提交");
      } catch (err) {
        console.warn("Step1 tracking failed", err);
      }
    }

    function showStep(step) {
      $(".reg-step").removeClass("is-active");
      $('.reg-step[data-step="' + step + '"]').addClass("is-active");
    }

    function applyRegSuccessQrcode(url) {
      if (!url) {
        return;
      }
      state.qrcodeUrl = url;
      $(".js-reg-qrcode").attr("src", url);
    }

    function applyRegSuccessQrcodeFallback() {
      $(".js-reg-qrcode").attr("src", REG_QRCODE_FALLBACK_URL);
    }

    function preloadRegSuccessQrcode() {
      if (!isDesktopForm()) {
        return;
      }
      if (state.qrcodeUrl) {
        applyRegSuccessQrcode(state.qrcodeUrl);
        return;
      }
      if (state.qrcodeLoading) {
        return;
      }
      if (!window.FsYxt || typeof window.FsYxt.generateQrcode !== "function") {
        applyRegSuccessQrcodeFallback();
        return;
      }
      state.qrcodeLoading = true;
      FsYxt.generateQrcode(REG_QRCODE_CONFIG.type, getRegQrcodeId(), {
        renderType: "manual",
        success: function (result) {
          state.qrcodeLoading = false;
          if (result && result.qrUrl) {
            applyRegSuccessQrcode(result.qrUrl);
          } else {
            applyRegSuccessQrcodeFallback();
          }
        },
        fail: function (err) {
          state.qrcodeLoading = false;
          console.warn("顾问二维码预生成失败，使用默认图", err);
          applyRegSuccessQrcodeFallback();
        },
      });
    }

    function showSuccessStep() {
      showStep(3);
      if (state.qrcodeUrl) {
        applyRegSuccessQrcode(state.qrcodeUrl);
      } else {
        preloadRegSuccessQrcode();
        if (!state.qrcodeUrl) {
          applyRegSuccessQrcodeFallback();
        }
      }
    }

    function toggleCheckbox($el) {
      $el.toggleClass("is-checked");
      $el.attr("aria-checked", $el.hasClass("is-checked"));
    }

    function validatePhone(phone) {
      return /^1\d{10}$/.test(phone);
    }

    function validateStep1() {
      clearFieldErrors();
      var valid = true;
      var phone = $.trim($(".js-reg-phone").val());
      var code = $.trim($(".js-reg-code").val());
      var company = $.trim($(".js-reg-company").val());

      if (!phone) {
        setEmptyFieldError("phone");
        valid = false;
      } else if (!validatePhone(phone)) {
        setValueFieldError("phone", "请输入正确的手机号");
        valid = false;
      }

      if (state.needVerifyCode) {
        if (!code) {
          setEmptyFieldError("code");
          valid = false;
        }
      }

      if (!company) {
        setEmptyFieldError("company");
        valid = false;
      }

      if (!$(".js-reg-agree").hasClass("is-checked")) {
        setValueFieldError("agree", "请阅读并勾选协议");
        valid = false;
      }

      if (!valid) {
        return null;
      }

      var payload = {
        phone: phone,
        companyName: company,
        name: "未填写",
      };
      if (state.needVerifyCode && code) {
        payload.phoneVerifyCode = code;
      }
      return payload;
    }

    function renderStep2Options(field) {
      var $container = $(".js-reg-options");
      $container.empty();
      if (!field || !field.options || !field.options.length) {
        $container.html('<p style="color:#91959e;font-size:13px;">暂无可选项</p>');
        return;
      }

      var options = field.options.slice().filter(function (opt) {
        return !/^(其它|其他)$/.test(opt.label);
      });
      var first = options.shift();
      var html = [];

      if (first) {
        html.push(
          '<div class="reg-option-item reg-option-item--full js-reg-option is-highlight' +
            (state.selectedOptions[first.value] ? " is-checked" : "") +
            '" data-value="' +
            first.value +
            '"><span class="reg-checkbox' +
            (state.selectedOptions[first.value] ? " is-checked" : "") +
            '"></span><span class="reg-option-item__text">' +
            first.label +
            "</span></div>"
        );
      }

      for (var i = 0; i < options.length; i += 3) {
        html.push('<div class="reg-option-row">');
        for (var j = i; j < i + 3 && j < options.length; j++) {
          var opt = options[j];
          html.push(
            '<div class="reg-option-item js-reg-option' +
              (state.selectedOptions[opt.value] ? " is-checked" : "") +
              '" data-value="' +
              opt.value +
              '"><span class="reg-checkbox' +
              (state.selectedOptions[opt.value] ? " is-checked" : "") +
              '"></span><span class="reg-option-item__text">' +
              opt.label +
              "</span></div>"
          );
        }
        html.push("</div>");
      }

      html.push(
        '<div class="reg-option-other js-reg-option js-reg-option-other-row' +
          (state.selectedOptions[OTHER_OPTION_VALUE] ? " is-checked" : "") +
          '" data-value="' +
          OTHER_OPTION_VALUE +
          '"><span class="reg-checkbox' +
          (state.selectedOptions[OTHER_OPTION_VALUE] ? " is-checked" : "") +
          '"></span><span class="reg-option-item__text">其它</span><input type="text" class="reg-input js-reg-other-input" placeholder="请输入" value="' +
          (state.otherText || "") +
          '" /></div>'
      );

      $container.html(html.join(""));
    }

    function buildStep2SubmitValues() {
      var values = [];
      Object.keys(state.selectedOptions).forEach(function (key) {
        if (!state.selectedOptions[key]) {
          return;
        }
        if (key === OTHER_OPTION_VALUE) {
          var otherVal = $.trim($(".js-reg-other-input").val());
          if (otherVal) {
            values.push("other:" + otherVal);
          }
          return;
        }
        values.push(key);
      });
      return values;
    }

    function findStep2Field(fields) {
      if (!fields || !fields.length) {
        return null;
      }
      for (var i = 0; i < STEP2_FIELD_API_NAMES.length; i += 1) {
        var apiName = STEP2_FIELD_API_NAMES[i];
        var field = fields.find(function (item) {
          return item.apiName === apiName;
        });
        if (field) {
          return field;
        }
      }
      return null;
    }

    function parseEnrollId(res, fullRes) {
      var sources = [res, fullRes, res && res.data, fullRes && fullRes.data];
      for (var i = 0; i < sources.length; i += 1) {
        var item = sources[i];
        if (item && item.enrollId) {
          return item.enrollId;
        }
      }
      return "";
    }

    function validateStep2() {
      clearStep2Errors();
      var apiName = state.step2Field && state.step2Field.apiName;
      if (!apiName) {
        setStep2ValueError("表单配置异常，请刷新页面重试");
        return null;
      }

      var otherVal = $.trim($(".js-reg-other-input").val());
      if (otherVal && !state.selectedOptions[OTHER_OPTION_VALUE]) {
        state.selectedOptions[OTHER_OPTION_VALUE] = true;
        $(".js-reg-option-other-row").addClass("is-checked");
        $(".js-reg-option-other-row .reg-checkbox").addClass("is-checked");
      }

      var values = buildStep2SubmitValues();
      if (!values.length) {
        setStep2ValueError(getFieldHelpText(state.step2Field) || "请至少选择一项");
        return null;
      }

      if (state.selectedOptions[OTHER_OPTION_VALUE] && !otherVal) {
        setStep2ValueError("请输入其它内容");
        return null;
      }

      state.otherText = otherVal;
      var multipleChoice = {};
      multipleChoice[apiName] = values;
      return { multipleChoice: multipleChoice };
    }

    function loadFieldDescriptions() {
      return new Promise(function (resolve, reject) {
        FsYxt.FormSDK.getFieldDescriptions({
          formId: state.formId,
          success: function (res) {
            console.log("formFields", res);
            state.formFields = (res && res.fields) || [];
            applyFormFieldHints();
            renderStep2Options(state.step2Field);
            resolve(res);
          },
          fail: function (err) {
            reject(err);
          },
        });
      });
    }

    function bindEvents() {
      $(".js-reg-agree").on("click keypress", function (e) {
        if (e.type === "keypress" && e.which !== 13 && e.which !== 32) return;
        toggleCheckbox($(this));
        clearFieldErrorByEl($(this));
      });

      $("#regFormCard").on(
        "input",
        ".js-reg-phone, .js-reg-code, .js-reg-company, .js-reg-other-input",
        function () {
          clearFieldErrorByEl($(this));
        }
      );

      $(".js-reg-send-code").on("click", function () {
        var $btn = $(this);
        if ($btn.prop("disabled") || state.codeCountdown > 0) return;
        clearFieldErrors();
        var phone = $.trim($(".js-reg-phone").val());
        if (!phone) {
          setEmptyFieldError("phone");
          return;
        }
        if (!validatePhone(phone)) {
          setValueFieldError("phone", "请输入正确的手机号");
          return;
        }
        $btn.addClass("is-loading").prop("disabled", true);
        FsYxt.FormSDK.sendSMCode({
          formId: state.formId,
          mobile: phone,
          success: function () {
            state.codeCountdown = 60;
            function tick() {
              if (state.codeCountdown <= 0) {
                $btn.text("验证码").removeClass("is-loading").prop("disabled", false);
                return;
              }
              $btn.text(state.codeCountdown + "s");
              state.codeCountdown -= 1;
              state.codeTimer = setTimeout(tick, 1000);
            }
            tick();
          },
          fail: function (err) {
            $btn.removeClass("is-loading").prop("disabled", false);
            alert((err && err.errMsg) || "验证码发送失败，请稍后重试");
          },
        });
      });

      $("#regFormCard").on("click", ".js-reg-option", function (e) {
        if ($(e.target).is(".js-reg-other-input")) return;
        var $item = $(this);
        var value = $item.data("value");
        var checked = !$item.hasClass("is-checked");
        $item.toggleClass("is-checked", checked);
        $item.find(".reg-checkbox").toggleClass("is-checked", checked);
        state.selectedOptions[value] = checked;
        clearStep2Errors();
      });

      $("#regFormCard").on("input focus", ".js-reg-other-input", function () {
        state.otherText = $(this).val();
        var otherVal = $.trim($(this).val());
        if (otherVal) {
          var $other = $(".js-reg-option-other-row");
          if (!$other.hasClass("is-checked")) {
            $other.addClass("is-checked");
            $other.find(".reg-checkbox").addClass("is-checked");
            state.selectedOptions[OTHER_OPTION_VALUE] = true;
          }
        }
        clearStep2Errors();
      });

      $(".js-reg-submit-step1").on("click", function () {
        var $btn = $(this);
        if ($btn.prop("disabled")) return;
        var payload = validateStep1();
        if (!payload) return;

        $btn.prop("disabled", true).text("正在提交...");
        state.step1Handled = false;

        function resetStep1Btn() {
          $btn.prop("disabled", false).text("立即提交");
        }

        function completeStep1(res, fullRes) {
          if (state.step1Handled) {
            return;
          }
          state.enrollId = parseEnrollId(res, fullRes);
          if (!state.enrollId) {
            return;
          }
          state.step1Handled = true;
          resetStep1Btn();
          showStep(2);
          fireStep1Tracking();
        }

        var submitPromise = FsYxt.FormSDK.submitForm({
          formId: state.formId,
          sceneType: 0,
          data: payload,
          success: function (res) {
            if (submitPromise && typeof submitPromise.then === "function") {
              return;
            }
            completeStep1(res);
            if (!state.step1Handled) {
              resetStep1Btn();
              alert("提交成功但未获取到报名 ID，请刷新页面重试");
            }
          },
          fail: function (err) {
            resetStep1Btn();
            alert((err && err.errMsg) || "提交失败，请稍后重试");
          },
        });

        if (submitPromise && typeof submitPromise.then === "function") {
          submitPromise.then(function (fullRes) {
            if (!fullRes || Number(fullRes.errCode) !== 0) {
              resetStep1Btn();
              alert((fullRes && fullRes.errMsg) || "提交失败，请稍后重试");
              return;
            }
            completeStep1(fullRes.data, fullRes);
            if (!state.step1Handled) {
              resetStep1Btn();
              alert("提交成功但未获取到报名 ID，请刷新页面重试");
            }
          });
        }
      });

      $(".js-reg-submit-step2").on("click", function () {
        var $btn = $(this);
        if ($btn.prop("disabled")) return;
        if (!state.enrollId) {
          alert("请先完成第一步提交");
          showStep(1);
          return;
        }
        var extraData = validateStep2();
        if (!extraData) return;

        $btn.prop("disabled", true).text("正在提交...");
        FsYxt.FormSDK.submitForm({
          formId: state.formId,
          enrollId: state.enrollId,
          sceneType: 0,
          data: extraData,
          success: function () {
            $btn.prop("disabled", false).text("立即提交");
            showSuccessStep();
          },
          fail: function (err) {
            $btn.prop("disabled", false).text("立即提交");
            alert((err && err.errMsg) || "提交失败，请稍后重试");
          },
        });
      });
    }

    var formBootstrapStarted = false;
    var regFormEventsBound = false;

    function startFormBootstrap() {
      if (!isDesktopForm()) {
        return;
      }
      if (formBootstrapStarted) {
        return;
      }
      formBootstrapStarted = true;
      state.formId = getFormId();
      applyCachedFormFieldHints();
      waitForFormSDK()
        .then(loadFieldDescriptions)
        .catch(function () {
          console.warn("注册表单 SDK 初始化失败");
          $(".js-reg-form-fields").removeClass("is-fields-pending");
        });
    }

    function initRegForm() {
      if (regFormEventsBound) {
        return;
      }
      regFormEventsBound = true;
      bindEvents();
    }

    function tryBootstrapRegForm() {
      if (typeof window.jQuery === "undefined") {
        window.setTimeout(tryBootstrapRegForm, 50);
        return;
      }
      window.jQuery(function () {
        if (!formBootstrapStarted) {
          startFormBootstrap();
        }
        initRegForm();
      });
    }

    tryBootstrapRegForm();
    window.addEventListener("load", tryBootstrapRegForm);
  })(typeof window.jQuery !== "undefined" ? window.jQuery : function (fn) {
    if (typeof fn === "function") {
      if (document.readyState === "loading") {
        document.addEventListener("DOMContentLoaded", fn);
      } else {
        fn();
      }
    }
  });
