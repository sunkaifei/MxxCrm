
    $(".fx-w-toolbar .after-tel").click(function () {
      if ($(".tel-details").css("display") == "none") {
        $(".tel-details").show("fast");
      } else {
        $(".tel-details").hide("fast");
      }
    });
    $(function () {
      var loginstatus = $.cookie('userstatus');
      if (loginstatus == '已登陆') {
        $(".fx-w-toolbar.login").css("display", "block");
        $(".fx-w-toolbar.before").css("display", "none");
      } else {
        $(".fx-w-toolbar.before").css("display", "block");
        $(".fx-w-toolbar.login").css("display", "none");
      }
        if(window.isCheckinPageBanner){
            var $banner = $('.j-fmcg-bannerbox');
            var fmcgTop = $banner.position().top;
            var $fmcgBanner = $($banner).find('.second-nav');
        }
        $('.j-goto-reg').click(function(e) {
            var form = $('.free-trial-wrap');
            var position = form.position();
            $('html,body').animate({
                scrollTop: position.top
            }, 500);
            e.stopPropagation();
            return false;
        });

        var isMobile =
            /Android|webOS|iPhone|iPod|BlackBerry|MIDP|SymbianOS|NOKIA|SAMSUNG|LG|NEC|TCL|Alcatel|BIRD|DBTEL|Dopod|PHILIPS|HAIER|LENOVO|MOT-|Nokia|SonyEricsson|SIE-|Amoi|ZTE/i.test(
                navigator.userAgent
            );
        // 微信二维码自动弹出控制
        var hasAutoShownQrcode = false; // 标记是否已经自动弹出过
        var windowHeight = $(window).height(); // 获取窗口高度
        
        // 窗口大小改变时更新窗口高度
        $(window).resize(function() {
            windowHeight = $(window).height();
        });
        
        $(window).scroll(function () {
            var scrollTop = $(document).scrollTop();
            if (scrollTop > 50) {
                if (loginstatus == '已登陆') {
                    $(".fx-w-toolbar.login").css("display", "block");
                    $(".fx-w-toolbar.before").css("display", "none");
                } else {
                    $(".fx-w-toolbar.before").css("display", "block");
                    $(".fx-w-toolbar.login").css("display", "none");
                }
            } else {
                if (loginstatus == '已登陆') {
                    $(".fx-w-toolbar.login").css("display", "block");
                    $(".fx-w-toolbar.before").css("display", "none");
                } else {
                    $(".fx-w-toolbar.before").css("display", "block");
                    $(".fx-w-toolbar.login").css("display", "none");
                }
            }
            
            // 首次滚动超过窗口高度时，自动弹出微信二维码
            if (!hasAutoShownQrcode && scrollTop > windowHeight) {
                // 找到当前显示的微信咨询按钮
                var $activeOnlineService = $(".fx-w-toolbar:visible .onlineService");
                if ($activeOnlineService.length > 0 && !$activeOnlineService.data("user-interacted")) {
                    hasAutoShownQrcode = true;
                    var $qrcodeWrapper = $activeOnlineService.find(".online-qw-wrapper");
                    // 显示二维码并添加自动显示标记
                    $qrcodeWrapper.css("display", "block").addClass("auto-show");
                }
            }
            
            if(window.isCheckinPageBanner){
                if(scrollTop >= fmcgTop){
                    $($fmcgBanner).css({
                        width: isMobile ? '100%' : '102%',
                        position:'fixed',
                        top: '56px'
                    })
                }else {
                    $($fmcgBanner).css({
                        width: isMobile ? '100%' : '1200px',
                        position:'absolute',
                        top: isMobile ? '0': '-30px'
                    })
                }
            }
        })
      $(".footer-i-collapse")
        .find(".collapse-item")
        .children(".collapse-title")
        .click(function() {
          let arrow = $(this).children(".arrow");
          let itemText = $(this).siblings(".collapse-content");
          if (itemText.hasClass("active")) {
            $(this).removeClass('collapse-title-open')
            itemText.removeClass("active");
            arrow.removeClass("up-arrow");
            arrow.addClass("down-arrow");
          } else {
            $('.footer-i-collapse .collapse-item .collapse-content').removeClass("active")
            $('.footer-i-collapse .collapse-item .arrow').addClass("down-arrow")
            $('.footer-i-collapse .collapse-item .collapse-title').removeClass("collapse-title-open")
            $(this).addClass('collapse-title-open')
            itemText.addClass("active");
            arrow.addClass("up-arrow");
            arrow.removeClass("down-arrow");
          }
        });
    });

  