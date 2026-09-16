
    /**
     * 根据各个页面的路径判断激活页面
     */
    let pages = {
      production: ["product-yx", "product-xs", "product-service", "product-outer", "product-inter", "product-weixin", 
      "plat-Paas", "plat-BI", "open-api","product-roi","product-omni","solutions-ai"],
      solutions: ["solutions-266", "solutions-297", "solutions-pro-service", "solutions-641", "solutions-819", "solutions-821", "solutions-981", "solutions-1000", 
      "solutions-1008", "solution-small-business-crm","domestic","overseas","solutions-electronic","solutions-structure",'solutions-consumer-e',
      "solutions-energy",'scene-service','product-member','sf-migrate'],
      example: ["fxcase"],
      success: ["service", "implement", "proadvantage", "datacenter", "college","securecenter",'use'],
      resource: ["whitepaper", "marketingActivity", "baike", "tech-tools", "zixun"],
      about: ["about-2", "about-influence", "about-join", "about-connect"],
      partner: ["channel-partner", "channel-marketing", "channel-delivered", "ecological-partners","channel-query","investment-policy"]
    }
    console.log(location.href);
    let href = location.href;
    Object.keys(pages).forEach(key => {
      let target = pages[key].find(page => {
        let curPageName;
        if (href[href.length - 1] === '/') {
          curPageName = href.substring(href.lastIndexOf('/', href.length - 2) + 1).replace(/\//, '').replace('.html', '');
        } else {
          curPageName = href.substring(href.lastIndexOf('/') + 1).replace(/\//, '').replace('.html', '');
        }
        return curPageName === page;
      })
      if (target) {
        $(`#${key} .menu-name`).addClass("selected")
      } else {
        $(`#${key} .menu-name`).removeClass("selected")
      }
    })

    $(function () {
      var isMobile =
        /Android|webOS|iPhone|iPod|BlackBerry|MIDP|SymbianOS|NOKIA|SAMSUNG|LG|NEC|TCL|Alcatel|BIRD|DBTEL|Dopod|PHILIPS|HAIER|LENOVO|MOT-|Nokia|SonyEricsson|SIE-|Amoi|ZTE/i.test(
          navigator.userAgent
        );

      window.isMobile = isMobile;
      // 以宽度为800px以上一种布局 800以下一种布局
      let viewportMobile = false
      if (window.innerWidth < 800) {
        viewportMobile = true
      }
      window.fsViewportMobile = viewportMobile
      if (isMobile) {
        // $('.mxxcrm-navigation__wrapper').hide();
        // $('.nav-content').show();
        $('.download-toolbar,.live-toolbar').show();
        $('.mobile-top').show();

        /**
         * 移动端展示下，移除多余web的元素
         */
        $("[mobile-remove]").remove();
      } else {
        /**
         * web端展示下，移除多余移动端的元素
         */
        $("[web-remove]").remove();
      }
    })

    $(function () {
      $(".header-i-collapse")
        .find(".collapse-item")
        .children(".collapse-title")
        .click(function () {
          let arrow = $(this).children(".arrow");
          let siblings = $(this).siblings(".collapse-content");
          if (siblings.hasClass("active")) {
            siblings.removeClass("active");
            arrow.removeClass("up-arrow");
            arrow.addClass("down-arrow");
          } else {
            siblings.addClass("active");
            arrow.addClass("up-arrow");
            arrow.removeClass("down-arrow");
          }
        });
    });

    function showNav(_this) {
      let nav = $(_this);
      let wrapper = $(".mobile-nav-content");
      if (nav.hasClass("icon-daohangmoren")) {
        nav.removeClass("icon-daohangmoren");
        nav.addClass("icon-daohangguanbi");
        wrapper.addClass("nav-open");
        $("body").addClass("mobile-nav-open");
      } else {
        nav.addClass("icon-daohangmoren");
        nav.removeClass("icon-daohangguanbi");
        wrapper.removeClass("nav-open");
        $("body").removeClass("mobile-nav-open");
      }
    }

    function openPage(url, isBlank) {
      if (isBlank) {
        window.open(url, '_blank');
      } else {
        window.open(url, '_self');
      }
    }

    function callphone(tel) {
      window.location.href = "tel:" + tel;
    }

    /**
     * 滚动条滚动后，首页头部导航阴影的显示与隐藏，三角箭头的显示与隐藏
     */
    $(window).scroll(function () {
      let top = $(this).scrollTop();
      if (top > 0) {
        $('.mxxcrm-navigation__wrapper').addClass('show-shadow');

      } else {
        $('.mxxcrm-navigation__wrapper').removeClass('show-shadow');
      }
      if (top > 57) {
        $('.selected').addClass('selected-hide');
      } else {
        $('.selected').removeClass('selected-hide');
      }
    });
    /**
     * 头部导航hover，三角箭头的显示和隐藏
     */
    // $('.nav-item').hover(function() {
    //   let index = $(this).index()
    //   if (index === 0 || index === 1 || index === 3) {
    //     $('.selected').addClass('hide-selected');
    //   } else if (index === 2) {
    //     $(this).addClass('hide-selected');
    //     $(this).next().addClass('hide-selected');
    //   } else {
    //     $(this).addClass('hide-selected');
    //   }
    // }, function() {
    //   $('.selected').removeClass('hide-selected');
    // })
    $('.nav-item').hover(function () {
      $('.selected').addClass('hide-selected');
    }, function () {
      $('.selected').removeClass('hide-selected');
    });

    $('.notice .icon-daohangguanbi').click(function () {
      $('.notice').remove();
    })
  