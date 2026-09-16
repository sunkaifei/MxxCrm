
    /*模块箭头hover状态*/
    function kuanicon(opt) {
      $(opt)
        .find(".index-link")
        .children("img")
        .attr("src", "/ap/wp-content/uploads/2019/01/arrow-hover.png");
      $(opt)
        .find(".yellow-jiantou")
        .children("img")
        .attr("src", "/ap/wp-content/uploads/2019/01/case-hover.png");
    }

    function outkuanicon(opt) {
      $(opt)
        .find(".index-link")
        .children("img")
        .attr("src", "/ap/wp-content/uploads/2019/01/arrow-default.png");
      $(opt)
        .find(".yellow-jiantou")
        .children("img")
        .attr("src", "/ap/wp-content/uploads/2019/01/case-default.png");
    }
  