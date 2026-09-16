import { createIconifyIcon } from '@vben-core/icons';

export * from '@vben-core/icons';

export const MdiKeyboardEsc = createIconifyIcon('mdi:keyboard-esc');

export const MdiWechat = createIconifyIcon('mdi:wechat');

export const MdiGithub = createIconifyIcon('mdi:github');

export const MdiGoogle = createIconifyIcon('mdi:google');

export const MdiQqchat = createIconifyIcon('mdi:qqchat');

/** lucide */

export const LucideArrowBigLeft = createIconifyIcon('lucide:arrow-big-left');

export const LucideArrowLeft = createIconifyIcon('lucide:arrow-left');

export const LucideChevronLeft = createIconifyIcon('lucide:chevron-left');

export const LucideChevronDown = createIconifyIcon('lucide:chevron-down');
export const LucideChevronUp = createIconifyIcon('lucide:chevron-up');

export const LucideTrash = createIconifyIcon('lucide:trash');
export const LucideTrash2 = createIconifyIcon('lucide:trash-2');

export const LucidePencil = createIconifyIcon('lucide:pencil');
export const LucidePencilOff = createIconifyIcon('lucide:pencil-off');

export const LucideNotebookPen = createIconifyIcon('lucide:notebook-pen');
export const LucideFilePenLine = createIconifyIcon('lucide:file-pen-line');

export const LucideInfo = createIconifyIcon('lucide:info');

export const LucideEye = createIconifyIcon('lucide:eye');
export const LucideCheck = createIconifyIcon('lucide:check');
export const LucideX = createIconifyIcon('lucide:x');
export const LucidePlus = createIconifyIcon('lucide:plus');
export const LucidePlusCircle = createIconifyIcon('lucide:plus-circle');
export const LucideEdit = createIconifyIcon('lucide:edit');
export const LucideLoader2 = createIconifyIcon('lucide:loader-2');
export const LucideXCircle = createIconifyIcon('lucide:x-circle');
export const LucideCheckCircle = createIconifyIcon('lucide:check-circle');
export const LucideMaximize2 = createIconifyIcon('lucide:maximize-2');
export const LucideMinimize2 = createIconifyIcon('lucide:minimize-2');
export const LucideArrowDownToLine = createIconifyIcon(
  'lucide:arrow-down-to-line',
);

export const LucideEllipsisVertical = createIconifyIcon(
  'lucide:ellipsis-vertical',
);
export const LucideEllipsis = createIconifyIcon('lucide:ellipsis');
export const LucideTag = createIconifyIcon('lucide:tag');
export const LucideLayers = createIconifyIcon('lucide:layers');
export const LucideLink = createIconifyIcon('lucide:link');
export const LucideUnlink = createIconifyIcon('lucide:unlink');
export const LucideUserPlus = createIconifyIcon('lucide:user-plus');
export const LucideSettings = createIconifyIcon('lucide:settings');

// CRM常用图标
export const LucideBuilding2 = createIconifyIcon('lucide:building-2');
export const LucidePhone = createIconifyIcon('lucide:phone');
export const LucideMail = createIconifyIcon('lucide:mail');
export const LucideMapPin = createIconifyIcon('lucide:map-pin');
export const LucideGlobe = createIconifyIcon('lucide:globe');
export const LucideSmartphone = createIconifyIcon('lucide:smartphone');
export const LucideCalendar = createIconifyIcon('lucide:calendar');
export const LucideClock = createIconifyIcon('lucide:clock');
export const LucideHistory = createIconifyIcon('lucide:history');
export const LucideMessageCircle = createIconifyIcon('lucide:message-circle');
export const LucideMessageSquare = createIconifyIcon('lucide:message-square');
export const LucideMoreHorizontal = createIconifyIcon('lucide:more-horizontal');

// 商机/合同详情常用
export const LucideDollarSign = createIconifyIcon('lucide:dollar-sign');
export const LucideTrendingUp = createIconifyIcon('lucide:trending-up');
export const LucideUser = createIconifyIcon('lucide:user');
export const LucideFileText = createIconifyIcon('lucide:file-text');

// 线索/跟进记录详情
export const LucideArrowRightLeft = createIconifyIcon(
  'lucide:arrow-right-left',
);
export const LucideTimer = createIconifyIcon('lucide:timer');
export const LucideTarget = createIconifyIcon('lucide:target');
export const LucideUpload = createIconifyIcon('lucide:upload');
export const LucideImageOff = createIconifyIcon('lucide:image-off');
export const LucideBanknote = createIconifyIcon('lucide:banknote');
export const LucideShoppingCart = createIconifyIcon('lucide:shopping-cart');
export const LucideTruck = createIconifyIcon('lucide:truck');

// 统计分析图标
export const LucideUsers = createIconifyIcon('lucide:users');
export const LucideWallet = createIconifyIcon('lucide:wallet');
export const LucidePieChart = createIconifyIcon('lucide:pie-chart');
export const LucideLayoutDashboard = createIconifyIcon(
  'lucide:layout-dashboard',
);
export const LucideArrowRight = createIconifyIcon('lucide:arrow-right');
export const LucideFileSignature = createIconifyIcon('lucide:file-signature');
export const LucideReceipt = createIconifyIcon('lucide:receipt');

// 文件管理图标
export const LucideGrid3x3 = createIconifyIcon('lucide:grid-3x3');
export const LucideList = createIconifyIcon('lucide:list');
export const LucideSearch = createIconifyIcon('lucide:search');
export const LucideImage = createIconifyIcon('lucide:image');
export const LucideDownload = createIconifyIcon('lucide:download');
export const LucideCopy = createIconifyIcon('lucide:copy');
export const LucideFolderOpen = createIconifyIcon('lucide:folder-open');
export const LucideFilm = createIconifyIcon('lucide:film');
export const LucideFile = createIconifyIcon('lucide:file');

// AI设置相关图标
export const LucideBot = createIconifyIcon('lucide:bot');
export const LucideEdit3 = createIconifyIcon('lucide:edit-3');
export const LucideSettings2 = createIconifyIcon('lucide:settings-2');
export const LucideLock = createIconifyIcon('lucide:lock');
export const LucideUnlock = createIconifyIcon('lucide:unlock');
export const LucideKeyRound = createIconifyIcon('lucide:key-round');

// 模板管理图标
export const LucideLayoutGrid = createIconifyIcon('lucide:layout-grid');
export const LucideMonitor = createIconifyIcon('lucide:monitor');
export const LucideTablet = createIconifyIcon('lucide:tablet');

/**
 * lucide 图标集中并没有 `display`，原本指向的是一个不存在的图标（渲染为空白）。
 * 这里复用语义相同的 monitor，保留导出名以兼容既有引用。
 *
 * @deprecated 请直接使用 LucideMonitor
 */
export const LucideDisplay = LucideMonitor;

// 通知公告图标
export const LucideSend = createIconifyIcon('lucide:send');
export const LucideUndo2 = createIconifyIcon('lucide:undo-2');
export const LucideMegaphone = createIconifyIcon('lucide:megaphone');

// 客户转移图标
export const LucidePackage = createIconifyIcon('lucide:package');
export const LucideScrollText = createIconifyIcon('lucide:scroll-text');
export const LucideUserCheck = createIconifyIcon('lucide:user-check');

// 盘点操作图标
export const LucidePlay = createIconifyIcon('lucide:play');
export const LucideSquare = createIconifyIcon('lucide:square');

// CRM 删除与作废图标
export const LucideCircleOff = createIconifyIcon('lucide:circle-off');

// ── 可视化 PDF 模板设计器（views/system/pdf-designer） ──────────────────────
// ⚠️ 本清单为手工维护的具名导出，新增图标必须在此登记，
//    否则 `@vben/icons` 无该成员（TS2305/TS2724），
//    且 scripts/gen-lucide-subset.mjs 扫描不到 lucide: 前缀字符串会漏进子集。
export const LucideAlignCenterHorizontal = createIconifyIcon(
  'lucide:align-center-horizontal',
);
export const LucideAlignCenterVertical = createIconifyIcon(
  'lucide:align-center-vertical',
);
export const LucideAlignEndHorizontal = createIconifyIcon(
  'lucide:align-end-horizontal',
);
export const LucideAlignEndVertical = createIconifyIcon(
  'lucide:align-end-vertical',
);
export const LucideAlignStartHorizontal = createIconifyIcon(
  'lucide:align-start-horizontal',
);
export const LucideAlignStartVertical = createIconifyIcon(
  'lucide:align-start-vertical',
);
export const LucideArrowDown = createIconifyIcon('lucide:arrow-down');
export const LucideArrowUp = createIconifyIcon('lucide:arrow-up');
export const LucideBarcode = createIconifyIcon('lucide:barcode');
export const LucideCircle = createIconifyIcon('lucide:circle');
// lucide 新版把 loader 更名为 loader-circle，这里指向新名以保证渲染
export const LucideLoader = createIconifyIcon('lucide:loader-circle');
export const LucideEyeOff = createIconifyIcon('lucide:eye-off');
export const LucideFileInput = createIconifyIcon('lucide:file-input');
export const LucideHash = createIconifyIcon('lucide:hash');
export const LucideListTree = createIconifyIcon('lucide:list-tree');
export const LucideMagnet = createIconifyIcon('lucide:magnet');
export const LucideMinus = createIconifyIcon('lucide:minus');
export const LucidePenTool = createIconifyIcon('lucide:pen-tool');
export const LucideQrCode = createIconifyIcon('lucide:qr-code');
export const LucideRedo2 = createIconifyIcon('lucide:redo-2');
export const LucideRepeat = createIconifyIcon('lucide:repeat');
export const LucideSave = createIconifyIcon('lucide:save');
export const LucideShapes = createIconifyIcon('lucide:shapes');
export const LucideShieldCheck = createIconifyIcon('lucide:shield-check');
export const LucideSparkles = createIconifyIcon('lucide:sparkles');
export const LucideStamp = createIconifyIcon('lucide:stamp');
export const LucideStar = createIconifyIcon('lucide:star');
export const LucideTable = createIconifyIcon('lucide:table');
export const LucideType = createIconifyIcon('lucide:type');
export const LucideVariable = createIconifyIcon('lucide:variable');
export const LucideZoomIn = createIconifyIcon('lucide:zoom-in');
export const LucideZoomOut = createIconifyIcon('lucide:zoom-out');
