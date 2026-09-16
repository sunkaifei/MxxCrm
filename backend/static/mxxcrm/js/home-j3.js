
  window.scrollToForm = function () {
    var anchor = document.getElementById("regFormCard") || document.getElementById("marketingFrame");
    if (anchor) {
      var headerHeight = 103;
      var top = anchor.getBoundingClientRect().top + window.pageYOffset - headerHeight;
      window.scrollTo({ top: top, behavior: "smooth" });
    }
  };

  $(function () {
    var mySwiper = new Swiper("#swiper1", {
      loop: true,
      slidesPerView: "auto",
      loopedSlides: 2,
      touchRatio: 0.5,
      pagination: {
        el: ".swiper-pagination",
        clickable: true,
        bulletClass: 'my-bullet',
        bulletActiveClass: 'my-bullet-active',
        modifierClass: 'my-pagination-'
      },
      autoplay: {
        delay: 6000,
      },
    });

    var mySwiper2 = new Swiper("#swiper2", {
      loop: false, // 循环模式选项
      slidesPerView: 1, //设置slider容器能够同时显示的slides数量
      autoplay: {
        delay: 5000,
      },
      on: {
        slidePrevTransitionStart: function (swiper) {
          $(".container-customer .next").addClass("direction-active");
          //左--无法向左
          if (mySwiper2.isBeginning) {
            $(".container-customer .prev").removeClass("direction-active");
          }
        },
        slideNextTransitionStart: function (swiper) {
          $(".container-customer .prev").addClass("direction-active");
          //向右-无法继续向右
          if (mySwiper2.isEnd) {
            $(".container-customer .next").removeClass("direction-active");
          }
        },
        slideChangeTransitionStart: function (swiper) {
          $(".indicator > div")
            .eq(swiper.activeIndex)
            .addClass("indicator-active")
            .siblings()
            .removeClass("indicator-active");
        },
      },
    });

    $(".container-customer .prev").click(function () {
      const swiperIndex = (window.innerWidth > 800) ? 0 : 1
      mySwiper2[swiperIndex].slidePrev();
    });
    $(".container-customer .next").click(function () {
      const swiperIndex = (window.innerWidth > 800) ? 0 : 1
      mySwiper2[swiperIndex].slideNext();
    });

    $(".indicator > div").click(function () {
      const swiperIndex = (window.innerWidth > 800) ? 0 : 1
      $(this)
        .addClass("indicator-active")
        .siblings()
        .removeClass("indicator-active");
      mySwiper2[swiperIndex].slideTo($(this).index());
    });

    // 移动端不轮播  手动滑动
    if (!(window.innerWidth < 800)) {
      var mySwiper3 = new Swiper("#swiper3", {
        loop: false, // 循环模式选项
        slidesPerView: 1, //设置slider容器能够同时显示的slides数量
        on: {
          slidePrevTransitionStart: function (swiper) {
            $(".container-scene .next").addClass("direction-active");
            //左--无法向左
            if (mySwiper3.isBeginning) {
              $(".container-scene .prev").removeClass("direction-active");
            }
          },
          slideNextTransitionStart: function (swiper) {
            $(".container-scene .prev").addClass("direction-active");
            //向右-无法继续向右
            if (mySwiper3.isEnd) {
              $(".container-scene .next").removeClass("direction-active");
            }
          },
        },
      });
    }
    $(".container-scene .prev").click(function () {
      mySwiper3.slidePrev();
    });
    $(".container-scene .next").click(function () {
      mySwiper3.slideNext();
    });

    if (!isMobile) {
      document.querySelectorAll("[web-auto]").forEach(function (video) {
        video.play();
      })
    }
    if (isMobile) {
      $(window).scroll(function () {
        var T = $("html").scrollTop();
        var headerNav = $('.nav-content');
        if (T > 0) {
          headerNav.css('background-color', '#fff')
        } else {
          headerNav.css('background-color', '#F0F4FB')
        }
      })
    };
  });

  // 移动端 iframe 表单（PC 端使用 FormSDK 自定义表单）
  function initSemMarketingIframe() {
    var devMode = window.origin.indexOf('ceshi112.com') !== -1;
    var mobileFrame = document.getElementById('marketingFrame');
    if (!mobileFrame || window.innerWidth > 800 || mobileFrame.src) {
      return;
    }
    mobileFrame.src = devMode
      ? 'https://www.ceshi112.com/proj/page/marketing-form?objectType=28&ea=88146&formId=cae8b8bc9bcf41d7b5ba93baeed00217&objectId=5a40dfe4f6014a94bcc29e05fd963b75&needReport=true'
      : 'https://www.mxxsaas.com/ec/h5-landing/release/index.html?id=251ce5a9f1a34edebf0f12532011cb8e&sourceType=7&type=1';
  }

  if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', initSemMarketingIframe);
  } else {
    initSemMarketingIframe();
  }
  window.addEventListener('load', initSemMarketingIframe);

