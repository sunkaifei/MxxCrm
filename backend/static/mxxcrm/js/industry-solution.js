class industrySolution extends HTMLElement {
  constructor() {
    super();
    const shadow = this.attachShadow({ mode: "open" });

    const doc = `
    <div class="solution">
      <h2 class="solution-title mobile-title">更多行业解决方案</h2>
      <div class="solutions mobile-bfc">
        <a href="/ap/solutions-981.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://a9.fspage.com/ap/wp-content/uploads/2021/03/7_【更多行业解决方案】IT互联网@2x.png"
                alt="IT互联网"
              />
            </div>
            <p class="item-title">ICT行业</p>
          </div>
        </a>
        <a href="/ap/solutions-819.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://a9.fspage.com/ap/wp-content/uploads/2021/03/7_【更多行业解决方案】快速消费品@2x.png"
                alt="快速消费品"
              />
            </div>
            <p class="item-title">快速消费品</p>
          </div>
        </a>
        <a href="/ap/solutions-266.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://a9.fspage.com/ap/wp-content/uploads/2021/03/7_【更多行业解决方案】机械制造@2x.png"
                alt="机械制造"
              />
            </div>
            <p class="item-title">工业自动化</p>
          </div>
        </a>
        <a href="/ap/solutions-297.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://a9.fspage.com/ap/wp-content/uploads/2021/03/7_【更多行业解决方案】软件与信息服务@2x.png"
                alt="软件与信息服务"
              />
            </div>
            <p class="item-title">SaaS软件行业</p>
          </div>
        </a>
        <a href="/ap/solutions-1008.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://www.mxxsaas.com/ap/wp-content/uploads/2022/03/7_【更多行业解决方案】家居建材@2x.png"
                alt="家居建材"
              />
            </div>
            <p class="item-title">家居建材</p>
          </div>
        </a>
        <a href="/ap/solutions-641.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://www.mxxsaas.com/ap/wp-content/uploads/2021/05/8_【更多行业解决方案】医疗@2x.png"
                alt="医疗器械"
              />
            </div>
            <p class="item-title">医疗健康</p>
          </div>
        </a>
        <a href="/ap/solutions-821.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://a9.fspage.com/ap/wp-content/uploads/2021/03/7_【更多行业解决方案】系统集成@2x.png"
                alt=""
              />
            </div>
            <p class="item-title">教育培训行业</p>
          </div>
        </a>
        <a href="/ap/solutions-1000.html" target="_blank">
          <div class="item mobile-width-100">
            <div>
              <img
                class="item-icon"
                src="https://www.mxxsaas.com/ap/wp-content/uploads/2022/01/农牧农资@2x-1.png"
                alt="农牧农资"
              />
            </div>
            <p class="item-title">农牧农资行业</p>
          </div>
        </a>
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
  
      .mobile-title {
        font-size: 24px !important;
        padding: 10px 20px !important;
        margin: 0px !important;
        text-align: center !important;
        line-height: 30px !important;
      }
  
      .mobile-width-100 {
        width: 100% !important;
      }
    }
    h2 {
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
    .solution {
      padding: 40px 0 24px 0;
      text-align: center;
    }
    .solution-title {
      font-size: 32px;
      font-weight: 500;
      color: #181c25;
      line-height: 42px;
      margin-bottom: 24px;
    }
    .solutions {
      width: 1200px;
      margin: auto;
      display: flex;
      flex-wrap: wrap;
      justify-content: space-around;
    }
    .item {
      width: 288px;
      height: 170px;
      background: #ffffff;
      border-radius: 4px;
      border: 1px solid #dee1e6;
      margin-bottom: 16px;
    }
    .item-icon {
      margin-top: 43px;
      margin-bottom: 22px;
      width: 48px;
      height: 48px;
      display: inline-block;
    }
    .item-title {
      font-size: 24px;
      font-weight: 500;
      color: #181c25;
      line-height: 34px;
    }
    `;

    shadow.innerHTML = doc;
    shadow.appendChild(style);
  }
}

customElements.define("industry-solution", industrySolution);
