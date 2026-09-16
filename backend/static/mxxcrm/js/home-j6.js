
  function getCookie(name) {
    var arr,
      reg = new RegExp('(^| )' + name + '=([^;]*)(;|$)');
    if ((arr = document.cookie.match(reg))) return unescape(arr[2]);
    else return null;
  }

  function setCookie(cookieName, cookieValue, year) {
    var today = new Date();
    var expire = new Date();
    expire.setTime(today.getTime() + year * 24 * 60 * 60 * 1000 * 365); // 
    document.cookie = cookieName + "=" + escape(cookieValue) + ";expires=" + expire.toGMTString() + ";path=/";
  }
  let isAllowCookie = getCookie('isAllowCookie')
  if (!isAllowCookie && window.innerWidth > 800) {
    $('.footer-cookie-wrapper').css('display', 'flex')
  }
  $("#cookie-btn-agree").click(function () {
    setCookie('isAllowCookie', true, 50)
    $('.footer-cookie-wrapper').css('animation', 'myfirst 0.5s')
    $('.footer-cookie-wrapper').css('-moz-animation', 'myfirst 0.5s')
    $('.footer-cookie-wrapper').css('-webkit-animation', 'myfirst 0.5s')
    $('.footer-cookie-wrapper').css('animation', 'myfirst 0.5s')
    setTimeout(function () {
      $('.footer-cookie-wrapper').css('display', 'none')
    }, 500)
  });

      /*
    * 全局alert
    */
    function fx_global_alert_function(timeout) {
      $('.fx-global-alert').css('display', 'flex')
      setTimeout(() => {
        $('.fx-global-alert').css('display', 'none')
      }, timeout || 3000);
    }
    function openPage(url, isBlank) {
    if (isBlank) {
      window.open(url);
    } else {
      window.open(url, '_self');
    }
  }
   
/**
 * 全局视频预览
 */
function fx_global_video_dialog(){
  $('.video-dialog-visible').click(event=>{
    var button = event.currentTarget // Button that triggered the modal
    var video = button.dataset.video;
    var title = button.dataset.title;
    var link = button.dataset.link;
    var poster = button.dataset.poster;
    $('.video-play-dialog').css('display','flex')
    if(poster){
      $('.video-play-dialog .play-videos').attr('poster',poster)
    }
    if(video){
      $('.video-play-dialog .play-videos').attr('src',video)
      $('.video-play-dialog .play-videos source').attr('src',video)
    }
    if(title){
      $('.video-play-dialog .dialog-video-title').text(title)
    }
    if(link){
      $('.video-play-dialog .dialog-video-btn').attr('href',link)
      $('.video-play-dialog .dialog-video-btn').css('display','inline-block')
    }
    $('.video-play-dialog .play-videos')[0].play()
  })
  $('.video-play-dialog-close').click(e=>{
    $('.video-play-dialog .play-videos').attr('src','')
    $('.video-play-dialog .play-videos source').attr('src','')
    $('.video-play-dialog').css('display','none')
  })
  $('.video-play-dialog .dialog-mask').click(e=>{
    $('.video-play-dialog .play-videos').attr('src','')
    $('.video-play-dialog .play-videos source').attr('src','')
    $('.video-play-dialog').css('display','none')
  })
}



;

  $('.footer_nav-item .company-list a').click(function () {
    var comindex = $(this).index();
    $(this).addClass("company-active").siblings().removeClass("company-active");
    if (comindex == 0) {
      $('.footer_nav-item .quyu').html("海淀区知春路甲63号卫星大厦7层");
    } else if (comindex == 1) {
      $('.footer_nav-item .quyu').html("上海市长宁区淞虹路377号 光大安石虹桥中心T2座北8层802、803室");
    } else if (comindex == 2) {
      $('.footer_nav-item .quyu').html("浙江省杭州市拱墅区储鑫路19号运河网谷10幢13楼1303");
    } else if (comindex == 3) {
      $('.footer_nav-item .quyu').html("深圳市铜鼓路39号大冲国际中心5号楼第22层");
    } else if (comindex == 4) {
      $('.footer_nav-item .quyu').html(" 广州市天河区天河北路235号广州环贸中心1501B单元");
    } else if (comindex == 5) {
      $('.footer_nav-item .quyu').html("南京市江宁区东山街道金源路2号绿地之窗商务广场D1幢1603");
    } else if (comindex == 6) {
      $('.footer_nav-item .quyu').html("福建省福州市台江区上浦路73号富力中心B1栋3014室");
    } else if (comindex == 7) {
      $('.footer_nav-item .quyu').html(
        `<span style="margin-bottom: 10px;display: inline-block;">四川省成都市武侯区锦城大道666号奥克斯广场B1006</span>
        <span>重庆市两江新区高新园星光大道60号金星科技大厦A区2-2（209）</span>
        `);
    } else if (comindex == 8) {
      $('.footer_nav-item .quyu').html("西安市雁塔区科创路168号西安电子科技大学科技园B座");
    } else if (comindex == 9) {
      $('.footer_nav-item .quyu').html("长沙市岳麓区梅溪湖金茂广场北塔17楼");
    } else if (comindex == 10) {
      $('.footer_nav-item .quyu').html("武汉市武昌区徐家棚街道宸胜国际中心2702-2室");
    } else if (comindex == 11) {
      $('.footer_nav-item .quyu').html("郑州市郑东新区金水东路美盛中心1605");
    } else if (comindex == 12) {
      $('.footer_nav-item .quyu').html("山东省济南市历下区中垠汇鑫时代中心3号楼301");
    } else if (comindex == 13) {
      $('.footer_nav-item .quyu').html("贵州省贵阳市观山湖区金阳街道林城西路摩根中心A座18层1号");
    } else if (comindex == 14) {
      $('.footer_nav-item .quyu').html("香港湾仔庄士敦道181号大有大厦7樓701室");
    } else if (comindex == 15) {
      $('.footer_nav-item .quyu').html(`
      <span style="margin-bottom: 10px;display: inline-block;">6200 Stoneridge Mall Road, Suite 300 Pleasanton, CA 94588</span>
      <span>400 Spectrum Center Drive Floor 19 Irvine, CA 92618</span>
      `);
    }
  });
