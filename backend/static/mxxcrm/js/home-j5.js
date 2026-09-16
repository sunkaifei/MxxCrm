
  //当点击跳转链接后，回到页面顶部位置
  $("#backTop").click(function () {
    $('body,html').animate({
      scrollTop: 0
    }, 500);
    return false;
  });

  $(function () {
    var loginstatus = $.cookie('userstatus');
    if (loginstatus == '已登陆') {
      $(".fx-w-toolbar.login").css("display", "block");
    } else {
      $(".fx-w-toolbar.before").css("display", "block");
    }
  });

  function _getHost(url) {
    var host = "";
    if (typeof url == undefined || url == null) return;
    var regex = /^\w+\:\/\/([^\/]*).*/;
    var match = url.match(regex);
    if (typeof match != "undefined" && match != null) {
      host = match[1];
    }
    return host;
  }

  $(function () {
    const imDialog = document.getElementsByClassName('service-details') ? document.getElementsByClassName('service-details')[0] : null
    const hasPopupImTips = sessionStorage.getItem('fs_hasPopupImTips') || ''
    const referrer_host = _getHost(document.referrer);
    if (hasPopupImTips != 'true' && (referrer_host !== window.location.host)) {
      sessionStorage.setItem('fs_hasPopupImTips', 'true')
      let count = 0
      const TIMER = setInterval(() => {
        const hasClickImWrapper = sessionStorage.getItem('fs_hasClickImWrapper') || ''
        if (count >= 15) {
          if (!hasClickImWrapper) {
            imDialog.style.display = 'flex'
          }
          clearInterval(TIMER)
        } else {
          count++
        }
      }, 1000);
    }
  })

  $('.service-close').click(e=>{
    if(e){
      e.stopPropagation()
    }
    const imDialog = document.getElementsByClassName('service-details') ? document.getElementsByClassName('service-details')[0] : null
    if (imDialog) {
      imDialog.style.display = 'none'
    }
  })


