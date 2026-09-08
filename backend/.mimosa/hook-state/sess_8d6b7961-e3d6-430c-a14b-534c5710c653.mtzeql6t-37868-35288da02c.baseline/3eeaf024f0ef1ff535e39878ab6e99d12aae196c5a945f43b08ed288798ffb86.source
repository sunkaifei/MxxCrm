class componySelect extends HTMLElement {
  constructor() {
    super();
    const shadow = this.attachShadow({ mode: "open" });

    const doc = `
    <div class="select mobile-bfc">
      <h2 class="select-title mobile-title"></h2>
      <div class="select-pic mobile-bfc">
        <img
          class="mobile-bfc"
          src=""
          alt=""
        />
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
    .select {
      padding: 40px 0 48px 0;
      text-align: center;
    }
    .select-title {
      font-size: 32px;
      font-weight: 500;
      color: #181c25;
      line-height: 42px;
      margin-bottom: 36px;
    }
    .select-pic {
      width: 1200px;
      margin: auto;
    }
    .select-pic img {
      width: 100%;
    }
    `;

    shadow.innerHTML = doc;
    shadow.appendChild(style);
    const title = this.getAttribute("title") || "更多行业企业的共同选择";
    const src = this.getAttribute("src");
    const alt = this.getAttribute("alt");
    shadow.querySelector(".select-title").innerHTML = title;
    shadow.querySelector("img").src = src;
    shadow.querySelector("img").alt = alt || "";
  }
}

customElements.define("compony-select", componySelect);
