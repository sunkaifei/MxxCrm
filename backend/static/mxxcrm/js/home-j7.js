
  $(function () {
    var isMobile =
      /Android|webOS|iPhone|iPod|BlackBerry|MIDP|SymbianOS|NOKIA|SAMSUNG|LG|NEC|TCL|Alcatel|BIRD|DBTEL|Dopod|PHILIPS|HAIER|LENOVO|MOT-|Nokia|SonyEricsson|SIE-|Amoi|ZTE/i.test(
        navigator.userAgent
      );

    if (isMobile) {
      initScale();
    }
    // 获取当前年份
    $('#fxcurrentYear').text(new Date().getFullYear())
    $('#fxcurrentYear1').text(new Date().getFullYear())
  })

  function initScale() {
    /**
     * 调节宽高
     */
    $(".mobile-scale-box [class^='mobile-scale']").each(function () {
      let scaleBox = $(this).parent(".mobile-scale-box");
      let rect = $(this).get(0).getBoundingClientRect();
      let width = rect.width;
      let height = rect.height;

      scaleBox.get(0).style.width = width + "px";
      scaleBox.get(0).style.height = height + "px";
    });
    /**
     * 只调节高度
     */
    $(".mobile-scale-height-box [class^='mobile-scale']").each(function () {
      let scaleHeightBox = $(this).parent(".mobile-scale-height-box");
      let rect = $(this).get(0).getBoundingClientRect();
      let height = rect.height;
      scaleHeightBox.get(0).style.height = height + "px";
    });
  }

  let observer = null;
  let resizeObserver = null;
  function patchResizeContainer() {
    // offset:容器自身的padding
    function smartLayout(container, resizeChildList, childType = 'fixed', gutter = 24, paddingOffset = 0,autoHeight = false) {
      if (!container || !resizeChildList) return
      // 自适应宽度
      let cellAdaptiveWidth = 0;
      // 子元素原本宽度
      let cellOriginWidth = 0;
      // 一共有几列（一行可以放下几个子元素）
      let originCellCols = 0;
      // 子元素初始宽度列表
      let cellOriginWidthList = []
      let domMaxHeight = 0
      const observerCallback = () => {
        if(window && window.innerWidth < 800) {
          return
        }
        if (childType === "fixed") {
          cellAdaptiveWidth = 0;
          cellOriginWidth = 0;
          originCellCols = 0;
        }
  
        if (childType === "auto") {
          cellOriginWidthList = [];
        }
        getCellOriginWidth();
      };
      const resizeObserverCallback = () => {
        if(window && window.innerWidth < 800) {
          return
        }
        getCellOriginWidth()
      };
  
      const getCellOriginWidth = () => {
        if (!resizeChildList) {
          return;
        }
  
        const childList = Array.from(resizeChildList)
        if (childList.length <= 1) {
          return;
        }
  
        childList.forEach((child) => {
          child.style.width = "";
          child.style.marginRight = "";
          child.style.marginBottom = "";
        });
  
        if (childType === "fixed") {
          const dom = childList[0];
          if (dom && dom.getBoundingClientRect) {
            const { width } = dom.getBoundingClientRect();
            cellOriginWidth = width;
            getCellAdaptiveWidth()
          }
        }
  
        if (childType === "auto") {
          cellOriginWidthList = childList.map((child) => {
            const dom = child;
            const { width,height} = dom.getBoundingClientRect();
            if(item.getAttribute('max-height') === 'true'){
              domMaxHeight = height
            }
            return width;
          });
          resetChildStyle()
        }
      };
  
      const getCellAdaptiveWidth = () => {
        if (!cellOriginWidth || !container) {
          return;
        }
        const containerRect = container.getBoundingClientRect();
        const containerWidth = containerRect.width - paddingOffset;
        let cellCols = Math.floor(
          (containerWidth + gutter) / (cellOriginWidth + gutter)
        );
        originCellCols = cellCols;
        const delta =
        containerWidth -
        originCellCols * (cellOriginWidth + gutter) +
        gutter;
        if (originCellCols <= 1) {
          cellAdaptiveWidth = containerWidth;
        }else{
          cellAdaptiveWidth = cellOriginWidth + delta / originCellCols;
        }
        const resetStyle = () => {
        const childList = Array.from(resizeChildList)
        childList.forEach((child, index) => {
          const isLastCol = (index + 1) % originCellCols === 0
          const rowCount = Math.ceil(childList.length / originCellCols)
          const isLastRow = index >= (rowCount - 1) * originCellCols
  
          /**
           * 虚拟DOM有可能尚未初始化好，child.elm为空，这里在下一个事件循环中多执行一次
           */
          if (child) {
            child.style.width = `${cellAdaptiveWidth}px`
            child.style.marginRight = `${isLastCol ? 0 : gutter}px`
            child.style.marginBottom = `${isLastRow ? 0 : gutter}px`
          }
        })
      }
      resetStyle()
      };
  
      const resetChildStyle = () => {
        if (!resizeChildList) {
          return;
        }
  
        const childList = Array.from(resizeChildList)
        if (childList.length !== cellOriginWidthList.length) {
          return;
        }
  
        const containerRect = container.getBoundingClientRect();
        const containerWidth = containerRect.width - paddingOffset;
  
        const cols = [];
        let startIndex = 0;
        let endIndex = 1;
        let colWidth = cellOriginWidthList[0];
  
        for (let i = 1; i < cellOriginWidthList.length; i += 1) {
          const cellWidth = cellOriginWidthList[i];
          if (cellWidth + colWidth + gutter > containerWidth) {
            cols.push({
              startIndex,
              endIndex,
              delta: containerWidth - colWidth,
              isLastCol: false,
            });
  
            startIndex = i;
            endIndex = i;
            colWidth = cellWidth;
            continue
          }
          colWidth += cellWidth + gutter;
          endIndex = i;
          if (i === cellOriginWidthList.length - 1) {
            cols.push({
              startIndex,
              endIndex,
              isLastCol: true,
            });
          }
        }
        cols.forEach((col) => {
          const { startIndex: start, endIndex: end, isLastCol, delta = 0 } = col;
          for (let i = start; i <= end; i += 1) {
            const count = end - start + 1;
  
            const child = childList[i];
            const width = cellOriginWidthList[i] + delta / count;
            const marginRight = i === end ? 0 : gutter;
            const marginBottom = isLastCol ? 0 : gutter;
            if(autoHeight){
              child.style.height = `${domMaxHeight}px`
            }
            child.style.width = `${width}px`;
            child.style.marginRight = `${marginRight}px`;
            child.style.marginBottom = `${marginBottom}px`;
          }
        });
      };
      observer = new MutationObserver(observerCallback);
      observer.observe(container, { childList: true });
      resizeObserver = new ResizeObserver(resizeObserverCallback);
      resizeObserver.observe(container);
    }
    const resizeContainer = document.querySelectorAll('.resize-container')
    for (let i = 0; i < resizeContainer.length; i++) {
      const childGutter = Number(resizeContainer[i].getAttribute('child-gutter'))
      const childResizeType = resizeContainer[i].getAttribute('child-type')
      const offset = Number(resizeContainer[i].getAttribute('self-offset'))
      const childList = resizeContainer[i].children
      const autoHeight = resizeContainer[i].getAttribute('auto-height') ? true : false
      smartLayout(resizeContainer[i], childList, childResizeType || 'fixed', childGutter || 24, offset || 0, autoHeight)
    }
}
patchResizeContainer()


  function touchToScaleImg(eleImg) {
    let initialDistance = 0;
    let initialScale = 1;

    eleImg.addEventListener('touchstart', (event) => {
      if (event.touches.length === 2) {
        initialDistance = getDistance(event.touches[0], event.touches[1]);
        initialScale = getCurrentScale(eleImg);
      }
    });

    eleImg.addEventListener('touchmove', (event) => {
       // 阻止默认的滚动行为
      if (event.touches.length === 2) {
        event.preventDefault();
        const currentDistance = getDistance(event.touches[0], event.touches[1]);
        const scale = (currentDistance / initialDistance) * initialScale;
        eleImg.style.transform = `scale(${scale})`;
      }
    },{ passive: false });

    function getDistance(touch1, touch2) {
      const dx = touch1.clientX - touch2.clientX;
      const dy = touch1.clientY - touch2.clientY;
      return Math.sqrt(dx * dy + dy * dy);
    }

    function getCurrentScale(element) {
      const transform = window.getComputedStyle(element).transform;
      if (transform === 'none') return 1;
      const matrix = transform.match(/^matrix\((.+)\)$/);
      return matrix ? parseFloat(matrix[1].split(', ')[0]) : 1;
    }

  }

  function clickToScaleImg() {
    let scale = 1;
    const previewWapper = document.querySelector('.fx-img-preview-wrapper')
    const previewImg = document.querySelector('.fx-img-preview-wrapper .fx-img-preview-main');
    const previewImgList = Array.from(document.querySelectorAll('#previewScaleImg'));
    $('.fx-img-preview-close').click(e => {
      // 关闭后重置scale
      scale = 1;
      $('.fx-img-preview-wrapper').css('display', 'none');
      $('.fx-img-preview-wrapper .fx-img-preview-main').attr('src', '');
      previewImg.style.transform = `scale(${scale}) translate(-50%,-50%)`;
    })
    $('.fx-img-preview-mask').click(e => {
      // 关闭后重置scale
      scale = 1;
      $('.fx-img-preview-wrapper').css('display', 'none');
      $('.fx-img-preview-wrapper .fx-img-preview-main').attr('src', '');
      previewImg.style.transform = `scale(${scale}) translate(-50%,-50%)`;
    })
    var isMobile =
      /Android|webOS|iPhone|iPod|BlackBerry|MIDP|SymbianOS|NOKIA|SAMSUNG|LG|NEC|TCL|Alcatel|BIRD|DBTEL|Dopod|PHILIPS|HAIER|LENOVO|MOT-|Nokia|SonyEricsson|SIE-|Amoi|ZTE/i.test(
        navigator.userAgent
      );
    previewImgList.forEach(item => {
      item.addEventListener('click', e => {
        if(!isMobile && !e.target.dataset.mobile){
          $('.fx-img-preview-wrapper').css('display', 'block');
          $('.fx-img-preview-wrapper .fx-img-preview-main').attr('src', e.currentTarget.currentSrc);
        }else if(isMobile){
          window.open(e.currentTarget.currentSrc || window.origin,'_blank')
        }
      })
    })
    if (!isMobile) {
      previewWapper.addEventListener('wheel', e => {
        e.preventDefault();
        const scaleAmount = 0.1;
        if (e.deltaY < 0) {
          scale += scaleAmount;
        } else {
          scale -= scaleAmount;
        }
        scale = Math.min(Math.max(0.5, scale), 3);
        previewImg.style.transform = `scale(${scale}) translate(-50%,-50%)`;
      });
    }
    const mousedown = (event) => {
      let innerX = event.clientX - previewImg.offsetLeft
      let innerY = event.clientY - previewImg.offsetTop
      // 移动时
      document.onmousemove = function (event) {
        previewImg.style.left = event.clientX - innerX + "px"
        previewImg.style.top = event.clientY - innerY + "px"
      }
      // 抬起时
      document.onmouseup = function () {
        document.onmousemove = null
        document.mousedown = null
        control()
      }
    }

    // 超出边界处理
    const control = function () {
      if (previewImg.offsetLeft < 0) {
        previewImg.style.left = 0 + "px"
      }

      if (previewImg.offsetTop < 0) {
        previewImg.style.top = 0 + "px"
      }

      if ((previewImg.offsetLeft + parseInt(previewImg.style.width)) > window.innerWidth) {
        previewImg.style.left = (window.innerWidth - parseInt(previewImg.style.width)) + "px"
      }

      if ((previewImg.offsetTop + parseInt(previewImg.style.height)) > window.innerHeight) {
        previewImg.style.top = (window.innerHeight - parseInt(previewImg.style.height)) + "px"
      }

    }
    // 按下时
    previewImg.addEventListener('mousedown', mousedown, false)
  }
  clickToScaleImg()

