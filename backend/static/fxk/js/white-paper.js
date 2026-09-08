class whitePaper extends HTMLElement {
  constructor() {
    super();
    const shadow = this.attachShadow({ mode: "open" });

    const doc = `
    <div class="wpaper__wbook mobile-bfc">
      <div class="wpaper__body">
        <div class="banner-cont mobile-bfc">
          <div class="banner-left mobile-bfc scale-08">
            <h1 class="title mobile-title"></h1>
            <div class="a1">
              <p class="sub-title"></p>
              <a
                target="_blank"
                class="free-btn"
                position="banner"
                href=""
                >免费下载</a
              >
            </div>
          </div>
          <div class="banner-img mobile-bfc">
            <img
              class="mobile-bfc"
              src=""
            />
          </div>
        </div>
      </div>
    </div>
  `;
    const style = document.createElement("style");
    style.textContent = `
    @media screen and (max-width: 480px) {
      .mobile-bfc {
        overflow: hidden;
        height: auto !important;
        min-height: auto !important;
        width: 100% !important;
        min-width: auto !important;
        margin: 0 !important;
        padding: 0 !important;
        display: block !important;
        position: static !important;
      }
  
      .scale-08 {
        transform: scale(0.8);
      }
  
      .mobile-title {
        font-size: 24px !important;
        padding: 10px 20px !important;
        margin: 0px !important;
        text-align: center !important;
        line-height: 30px !important;
      }
    }
    h1 {
      font-size: 16px;
      font-weight: normal;
      margin: 0;
    }
    p {
      margin: 0;
    }
    a {
      text-decoration: none;
    }
    .wpaper__wbook {
      padding-top: 104px;
    }

    .wpaper__wbook .wpaper__body{
      background-color: #fafcff;
      display: inline-block;
      position: relative;
      background-image: url("https://www.fxiaoke.com/ap/wp-content/uploads/2021/04/1_【banner】背景底纹@2x-1.png");
      background-size: cover;
      background-position: 50%;
      display: flex;
      align-items: center;
      justify-content: center;
    }

    .wpaper__wbook .banner-cont {
      width: 1200px;
      height: 276px;
      display: flex;
      justify-content: space-between;
      position: relative;
    }

    .wpaper__wbook .banner-cont .banner-left {
      display: flex;
      flex-direction: column;
      justify-content: center;
      width: 600px;
    }

    .wpaper__wbook .title {
      font-size: 36px;
      font-weight: 500;
      color: #181c25;
      line-height: 46px;
      margin-bottom: 12px;
    }

    .wpaper__wbook .sub-title {
      font-size: 20px;
      font-weight: 400;
      color: #181c25;
      line-height: 30px;
      margin-bottom: 24px;
    }

    .wpaper__wbook .banner-cont .banner-left .a1 {
      margin-left: 18px;
    }
    .wpaper__wbook .banner-cont .banner-img {
      position: absolute;
      bottom: 0;
      left: 756px;
    }

    .wpaper__wbook .banner-cont .banner-img img {
      width: 250px;
      height: 340px;
    }
    .free-btn {
      display: inline-block;
      width: 152px;
      height: 48px;
      background: linear-gradient(45deg, #ff704f 0%, #ff9326 100%);
      box-shadow: 0px 2px 12px 0px rgba(255, 128, 0, 0.12);
      border-radius: 24px;
      line-height: 48px;
      font-size: 24px;
      font-weight: 500;
      color: #ffffff;
      text-align: center;
    }
  `;

    shadow.innerHTML = doc;
    shadow.appendChild(style);
    const title = this.getAttribute("title");
    const subTitle = this.getAttribute("sub-title");
    const src = this.getAttribute("src");
    const pic = this.getAttribute("pic");
    shadow.querySelector(".title").innerHTML = title;
    shadow.querySelector(".sub-title").innerHTML = subTitle || "";
    shadow.querySelector(".free-btn").href = src;
    shadow.querySelector("img").src = pic;
  }
}

customElements.define("white-paper", whitePaper);
