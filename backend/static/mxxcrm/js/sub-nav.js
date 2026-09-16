/**
 * Web Component
 */
 class subNav extends HTMLElement {
  constructor() {
    super();
    this.attachShadow({ mode: "open" });

    this.parseItems();
  }

  attributeChangedCallback(name, oldValue, newValue) {
    this.parseItems();
  }

  static get observedAttributes() {
    return ["items"];
  }

  parseItems() {
    let items = this.getAttribute("items");
    if (!items) return;
    items = JSON.parse(items);
    let doc = `
    <div class="tab-navbar mobile-bfc mobile-scroll-x">
      <div class="tab-sub">
        <div class="tab-container">
    `;
    items.forEach((item) => {
      doc += `<a class="tab-nav-item ${
        item.title === this.getAttribute("selected") ? "selected" : ""
      }" href="${item.url}">${item.title}</a>`;
    });
    doc += `
        </div>
      </div>
    </div>
    `;
    this.shadowRoot.innerHTML = doc;

    const style = document.createElement("style");
    style.textContent = `
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
    .tab-navbar {
      height: 56px;
      width: 100%;
      border-top: 1px solid #eee;
    }

    .tab-navbar .tab-sub {
      width: 1200px;
      margin: auto;
      margin-top: 14px;
    }

    .tab-navbar .tab-container {
      display: flex;
      width: 1400px;
      align-items: center;
    }

    .tab-navbar .tab-container .tab-nav-item {
      display: block;
      box-sizing: border-box;
      padding: 0px 16px;
      font-size: 14px;
      margin-right: 12px;
      color: #181c25;
      height: 28px;
      line-height: 28px
    }

    .tab-navbar .tab-container .selected {
      background: #fff3eb;
      border-radius: 20px;
      color: #ff8000;
    }
  `;
    this.shadowRoot.appendChild(style);
  }
}

customElements.define("sub-nav", subNav);

/**
 * 导航数据
 */
const productionItems = [
  {
    title: "营销管理",
    url: "/ap/product-yx/",
  },
  {
    title: "销售管理",
    url: "/ap/product-xs/",
  },

  {
    title: "服务管理",
    url: "/ap/product-service/",
  },
  {
    title: "连接渠道",
    url: "/ap/product-outer/",
  },
  {
    title: "连接全员业务协同",
    url: "/ap/product-inter/",
  },
  {
    title: "连接生态和系统",
    url: "/ap/product-weixin/",
  },
  {
    title: "业务定制平台 (PaaS)",
    url: "/ap/plat-Paas/",
  },
  {
    title: "智能分析平台 (BI)",
    url: "/ap/plat-BI/",
  },
  {
    title: "数据集成平台+开放平台",
    url: "/ap/open-api/",
  },
];
let exampleItems = [
  {
    title: "ICT行业",
    url: "/ap/solutions-981.html",
  },
  {
    title: "医疗健康",
    url: "/ap/solutions-641.html",
  },
  {
    title: "SaaS软件",
    url: "/ap/solutions-297.html",
  },
  {
    title: "家居建材",
    url: "/ap/solutions-1008.html",
  },
  {
    title: "快消品行业",
    url: "/ap/solutions-819.html",
  },
  {
    title: "教育培训",
    url: "/ap/solutions-821.html",
  },
  {
    title: "工业自动化",
    url: "/ap/solutions-266.html",
  },
  {
    title: "农牧农资",
    url: "/ap/solutions-1000.html",
  },
];
let successItems = [
  {
    title: "服务体系",
    url: "/ap/service/",
  },
  {
    title: "客户实施服务",
    url: "/ap/implement",
  },
  {
    title: "安全保障",
    url: "/ap/proadvantage/",
  },
  {
    title: "用户手册",
    url: "/ap/datacenter",
  },
  {
    title: "迈客学院",
    url: "/ap/college",
  },
];
let resourceItems = [
  {
    title: "白皮书下载",
    url: "/ap/whitepaper",
  },
  {
    title: "市场活动",
    url: "/ap/marketingActivity",
  },
  {
    title: "产品动态",
    url: "/crm/baike",
  },
  {
    title: "数字化小工具",
    url: "/crm/tech-tools",
  },
  {
    title: "行业资讯",
    url: "/crm/zixun",
  },
];
let aboutItems = [
  {
    title: "企业简介",
    url: "/crm/about-2/",
  },
  {
    title: "迈客动态",
    url: "/crm/about-influence/",
  },
  {
    title: "加入迈客",
    url: "/crm/about-join/",
  },
  {
    title: "联系方式",
    url: "/crm/about-connect/",
  },
];

if (document.querySelector("#sub-nav-production"))
  document
    .querySelector("#sub-nav-production")
    .setAttribute("items", JSON.stringify(productionItems));
if (document.querySelector("#sub-nav-example"))
  document
    .querySelector("#sub-nav-example")
    .setAttribute("items", JSON.stringify(exampleItems));
if (document.querySelector("#sub-nav-success"))
  document
    .querySelector("#sub-nav-success")
    .setAttribute("items", JSON.stringify(successItems));
if (document.querySelector("#sub-nav-resource"))
  document
    .querySelector("#sub-nav-resource")
    .setAttribute("items", JSON.stringify(resourceItems));
if (document.querySelector("#sub-nav-about"))
  document
    .querySelector("#sub-nav-about")
    .setAttribute("items", JSON.stringify(aboutItems));
