//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

//! 文字点选验证码：布点/绘制/校验纯函数库（批2 后新增，方案见
//! docs/09-认证登录与权限/文字点选验证码开发方案.md）
//!
//! 设计要点：
//! - 坐标只在服务端留存，图片对外仅含"背景 + 散布汉字"；
//! - 背景程序化生成（渐变 + 干扰圆/线），每次不同，无模板可建库；
//! - 字符用 ab_glyph 按系统 CJK 字体栅格化，随机字号/旋转/颜色，零坐标泄露；
//! - 九宫格布点保证最小间距，避免点击歧义。

use image::{Rgba, RgbaImage};
use rand::Rng;

/// 点选会话尺寸（原图坐标基准；前端按显示尺寸换算回原图）
pub const IMAGE_W: u32 = 320;
pub const IMAGE_H: u32 = 160;
/// 点选字符数
pub const CHAR_COUNT: usize = 3;

/// 常用汉字字符集（避开生僻字与 OCR 高混淆字符）
pub const CHARSET: &[char] = &[
    '湖','光','山','色','潭','面','桃','花','林','木','风','云','天','地','海','河',
    '春','夏','秋','冬','晨','夕','雪','雨','露','霜','松','柏','竹','梅','兰',
    '江','桥','岛','峰','岩','泉','溪','洞','石','沙','城','村','街','巷','园',
    '东','南','西','北','中','上','下','左','右','前','后','高','低','长','短',
    '红','橙','黄','绿','青','蓝','紫','黑','白','灰','金','银','铜','铁','玉',
    '龙','凤','鹤','雁','莺','燕','蝶','蜂','鱼','虾','猫','犬','马','牛','羊',
    '书','画','琴','棋','诗','酒','茶','墨','纸','笔','剑','弓','旗','鼓','钟',
    '日','月','星','辰','云','霞','虹','雾','烟','波','涛','潮','流','浪','舟',
];

/// 形近字组：同组字符不得同时出现在一张图上（避免点击歧义）
const HOMOGLYPH_GROUPS: &[&[char]] = &[
    &['己', '已', '巳'],
    &['日', '曰'],
    &['天', '夫'],
    &['木', '本', '末', '术'],
    &['土', '士'],
    &['千', '干', '于'],
    &['人', '入', '八'],
    &['刀', '力'],
    &['王', '玉', '主'],
    &['晴', '精', '清', '请', '情'],
];

/// 选取干扰字符：与目标字符互不相同即可（允许与目标形近以增加识别难度，但不允许重复）
pub fn pick_distractors(targets: &[char], n: usize) -> Vec<char> {
    let mut rng = rand::thread_rng();
    let mut picked: Vec<char> = Vec::new();
    while picked.len() < n {
        let c = CHARSET[rng.gen_range(0..CHARSET.len())];
        if targets.contains(&c) || picked.contains(&c) {
            continue;
        }
        picked.push(c);
    }
    picked
}

/// 从字符集随机取 n 个互不相同、且不同属任一形近组的字符
pub fn pick_chars(n: usize) -> Vec<char> {
    let mut rng = rand::thread_rng();
    loop {
        let mut picked: Vec<char> = Vec::with_capacity(n);
        while picked.len() < n {
            let c = CHARSET[rng.gen_range(0..CHARSET.len())];
            if picked.contains(&c) {
                continue;
            }
            // 形近组互斥：与已选字符同组的拒绝
            let conflicts = HOMOGLYPH_GROUPS
                .iter()
                .any(|g| g.contains(&c) && picked.iter().any(|p| g.contains(p)));
            if conflicts {
                continue;
            }
            picked.push(c);
        }
        return picked;
    }
}

/// 九宫格布点：n 个点落在 n 个不同宫格内（中心 + 宫格 1/4 范围抖动），
/// 保证任意两点最小间距 ≥ min_dist
pub fn place_points(w: u32, h: u32, n: usize, min_dist: f64) -> Vec<(f64, f64)> {
    let mut rng = rand::thread_rng();
    let (fw, fh) = (w as f64, h as f64);
    let cols = 3.0_f64;
    let rows = ((n as f64) / cols).ceil().max(1.0);
    let cw = fw / cols;
    let ch = fh / rows;

    let mut cells: Vec<(usize, usize)> = Vec::new();
    for r in 0..rows as usize {
        for c in 0..cols as usize {
            cells.push((r, c));
        }
    }
    // 洗牌取前 n 个宫格
    for i in (1..cells.len()).rev() {
        let j = rng.gen_range(0..=i);
        cells.swap(i, j);
    }
    cells.truncate(n);

    loop {
        let pts: Vec<(f64, f64)> = cells
            .iter()
            .map(|&(r, c)| {
                let cx = c as f64 * cw + cw / 2.0;
                let cy = r as f64 * ch + ch / 2.0;
                let jx = rng.gen_range(-cw / 4.0..cw / 4.0);
                let jy = rng.gen_range(-ch / 4.0..ch / 4.0);
                (cx + jx, cy + jy)
            })
            .collect();
        let ok = (0..pts.len()).all(|i| {
            (0..pts.len()).all(|j| {
                i == j || {
                    let dx = pts[i].0 - pts[j].0;
                    let dy = pts[i].1 - pts[j].1;
                    (dx * dx + dy * dy).sqrt() >= min_dist
                }
            })
        });
        if ok {
            return pts;
        }
    }
}

/// 校验点击序列：数量一致 + 顺序一致 + 每点落在容差圆内
pub fn verify_clicks(
    points: &[(f64, f64)],
    clicks: &[(f64, f64)],
    tolerance: f64,
) -> Result<(), String> {
    if clicks.len() != points.len() {
        return Err(format!("需按顺序点击 {} 个文字", points.len()));
    }
    for (i, (px, py)) in points.iter().enumerate() {
        let (cx, cy) = clicks[i];
        let dx = cx - px;
        let dy = cy - py;
        if (dx * dx + dy * dy).sqrt() > tolerance {
            return Err(format!("第 {} 个文字点选位置不正确", i + 1));
        }
    }
    Ok(())
}

/// 程序化生成背景：随机双色渐变 + 半透明圆斑与细线（无固定模板，防建库）
pub fn gen_background(w: u32, h: u32) -> RgbaImage {
    let mut rng = rand::thread_rng();
    let mut img = RgbaImage::new(w, h);
    let (r1, g1, b1) = (
        rng.gen_range(180..=245),
        rng.gen_range(180..=245),
        rng.gen_range(180..=245),
    );
    let (r2, g2, b2) = (
        rng.gen_range(140..=220),
        rng.gen_range(140..=220),
        rng.gen_range(140..=220),
    );
    for y in 0..h {
        let t = y as f32 / h as f32;
        for x in 0..w {
            let tx = x as f32 / w as f32;
            let k = (t + tx) / 2.0;
            let r = (r1 as f32 + (r2 as f32 - r1 as f32) * k) as u8;
            let g = (g1 as f32 + (g2 as f32 - g1 as f32) * k) as u8;
            let b = (b1 as f32 + (b2 as f32 - b1 as f32) * k) as u8;
            img.put_pixel(x, y, Rgba([r, g, b, 255]));
        }
    }
    // 干扰圆斑（半透明，数量提升）
    for _ in 0..(7 + rng.gen_range(0..4)) {
        let (cx, cy, rad) = (
            rng.gen_range(0..w as i32),
            rng.gen_range(0..h as i32),
            rng.gen_range(12..36),
        );
        let alpha = rng.gen_range(20..55);
        let color = Rgba([rng.gen_range(60..160), rng.gen_range(60..160), rng.gen_range(80..180), alpha]);
        for y in (cy - rad).max(0)..=(cy + rad).min(h as i32 - 1) {
            for x in (cx - rad).max(0)..=(cx + rad).min(w as i32 - 1) {
                let dx = (x - cx) as f32 / rad as f32;
                let dy = (y - cy) as f32 / rad as f32;
                if dx * dx + dy * dy <= 1.0 {
                    blend_pixel(&mut img, x as u32, y as u32, color);
                }
            }
        }
    }
    // 干扰细线（直线数量提升）
    for _ in 0..6 {
        let (x1, y1) = (rng.gen_range(0..w), rng.gen_range(0..h));
        let (x2, y2) = (rng.gen_range(0..w), rng.gen_range(0..h));
        let color = Rgba([rng.gen_range(50..140), rng.gen_range(50..140), rng.gen_range(60..170), 110]);
        draw_line(&mut img, x1, y1, x2, y2, color);
    }
    // 正弦波干扰曲线（两段，穿越全图）
    for _ in 0..2 {
        let base_y = rng.gen_range(20..h as i32 - 20);
        let amp = rng.gen_range(6..18) as f32;
        let freq = rng.gen_range(0.04..0.09) as f32;
        let phase = rng.gen_range(0.0..6.28) as f32;
        let color = Rgba([rng.gen_range(40..130), rng.gen_range(50..140), rng.gen_range(70..170), 130]);
        let mut prev_x = 0u32;
        let mut prev_y = base_y.max(0) as u32;
        for x in 1..w {
            let yf = base_y as f32 + amp * ((x as f32 * freq) + phase).sin();
            let y = yf.round().clamp(0.0, (h - 1) as f32) as u32;
            draw_line(&mut img, prev_x, prev_y, x, y, color);
            prev_x = x;
            prev_y = y;
        }
    }
    img
}

fn blend_pixel(img: &mut RgbaImage, x: u32, y: u32, c: Rgba<u8>) {
    let p = img.get_pixel_mut(x, y);
    let a = c.0[3] as f32 / 255.0;
    let inv = 1.0 - a;
    *p = Rgba([
        (c.0[0] as f32 * a + p.0[0] as f32 * inv) as u8,
        (c.0[1] as f32 * a + p.0[1] as f32 * inv) as u8,
        (c.0[2] as f32 * a + p.0[2] as f32 * inv) as u8,
        255,
    ]);
}

fn draw_line(img: &mut RgbaImage, x1: u32, y1: u32, x2: u32, y2: u32, c: Rgba<u8>) {
    let (mut x, mut y) = (x1 as i64, y1 as i64);
    let (x2, y2) = (x2 as i64, y2 as i64);
    let dx = (x2 - x).abs();
    let dy = -(y2 - y).abs();
    let (mut sx, mut sy) = (if x < x2 { 1 } else { -1 }, if y < y2 { 1 } else { -1 });
    let mut err = dx + dy;
    loop {
        if x < img.width() as i64 && y < img.height() as i64 {
            blend_pixel(img, x as u32, y as u32, c);
        }
        if x == x2 && y == y2 {
            break;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

/// 旋转合成（最近邻，绕中心）：把字符贴片旋转后贴到背景上
pub fn rotate_and_paste(bg: &mut RgbaImage, tile: &RgbaImage, center: (f64, f64), deg: f64) {
    let (tw, th) = (tile.width() as f64, tile.height() as f64);
    let rad = deg.to_radians();
    let (sin, cos) = (rad.sin(), rad.cos());
    let (bg_w, bg_h) = (bg.width() as i64, bg.height() as i64);
    for ty in 0..th as i64 {
        for tx in 0..tw as i64 {
            let src = *tile.get_pixel(tx as u32, ty as u32);
            if src.0[3] == 0 {
                continue;
            }
            // 贴片中心坐标 → 旋转 → 回背景坐标
            let dx = tx as f64 - tw / 2.0;
            let dy = ty as f64 - th / 2.0;
            let bx = (center.0 + dx * cos - dy * sin).round() as i64;
            let by = (center.1 + dx * sin + dy * cos).round() as i64;
            if bx >= 0 && by >= 0 && bx < bg_w && by < bg_h {
                blend_pixel(bg, bx as u32, by as u32, src);
            }
        }
    }
}

/// 水平缩放（最近邻重采样）：以中心为基准，模拟字体扭曲
pub fn scale_tile_x(tile: &RgbaImage, factor: f64) -> RgbaImage {
    let (tw, th) = (tile.width(), tile.height());
    let new_w = ((tw as f64) * factor).round().max(1.0) as u32;
    let mut out = RgbaImage::new(new_w, th);
    for y in 0..th {
        for x in 0..new_w {
            let sx = ((x as f64 + 0.5) / factor - 0.5).round() as i32;
            if sx >= 0 && (sx as u32) < tw {
                out.put_pixel(x, y, *tile.get_pixel(sx as u32, y));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_chars_distinct_and_no_homoglyph() {
        for _ in 0..50 {
            let chars = pick_chars(CHAR_COUNT);
            assert_eq!(chars.len(), CHAR_COUNT);
            for i in 0..chars.len() {
                for j in i + 1..chars.len() {
                    assert_ne!(chars[i], chars[j]);
                    // 不同形近组
                    let same_group = HOMOGLYPH_GROUPS
                        .iter()
                        .any(|g| g.contains(&chars[i]) && g.contains(&chars[j]));
                    assert!(!same_group, "形近字同图: {:?} {:?}", chars[i], chars[j]);
                }
            }
        }
    }

    #[test]
    fn test_place_points_min_dist() {
        let pts = place_points(IMAGE_W, IMAGE_H, CHAR_COUNT, 90.0);
        assert_eq!(pts.len(), CHAR_COUNT);
        for i in 0..pts.len() {
            for j in i + 1..pts.len() {
                let d = ((pts[i].0 - pts[j].0).powi(2) + (pts[i].1 - pts[j].1).powi(2)).sqrt();
                assert!(d >= 90.0, "间距不足: {}", d);
            }
        }
        // 全部在图内（留 20px 边距给字形）
        for (x, y) in &pts {
            assert!(*x >= 20.0 && *x <= IMAGE_W as f64 - 20.0);
            assert!(*y >= 20.0 && *y <= IMAGE_H as f64 - 20.0);
        }
    }

    #[test]
    fn test_verify_clicks() {
        let pts = vec![(100.0, 80.0), (200.0, 40.0), (260.0, 120.0)];
        // 正确顺序 + 容差内
        assert!(verify_clicks(&pts, &[(102.0, 78.0), (198.0, 45.0), (262.0, 118.0)], 15.0).is_ok());
        // 顺序错
        assert!(verify_clicks(&pts, &[(200.0, 40.0), (100.0, 80.0), (260.0, 120.0)], 15.0).is_err());
        // 数量错
        assert!(verify_clicks(&pts, &[(100.0, 80.0)], 15.0).is_err());
        // 超容差
        assert!(verify_clicks(&pts, &[(130.0, 80.0), (200.0, 40.0), (260.0, 120.0)], 15.0).is_err());
    }

    #[test]
    fn test_gen_background_size() {
        let img = gen_background(IMAGE_W, IMAGE_H);
        assert_eq!(img.width(), IMAGE_W);
        assert_eq!(img.height(), IMAGE_H);
    }
}

// ==================== 渲染（字体加载 + 合成）====================

use ab_glyph::{Font, FontVec, Glyph, PxScale};
use std::sync::OnceLock;

static FONT: OnceLock<FontVec> = OnceLock::new();

/// 加载 CJK 字体（依次尝试候选路径；ttc 集合取 index 0）。已加载则跳过。
pub fn load_font(paths: &[String]) -> Result<(), String> {
    if FONT.get().is_some() {
        return Ok(());
    }
    for path in paths {
        let p = path.trim();
        if p.is_empty() {
            continue;
        }
        if let Ok(data) = std::fs::read(p) {
            for index in 0..2usize {
                if let Ok(font) = FontVec::try_from_vec_and_index(data.clone(), index as u32) {
                    let _ = FONT.set(font);
                    return Ok(());
                }
            }
        }
    }
    Err(format!("未找到可用的 CJK 字体文件，尝试过: {:?}", paths))
}

/// 合成点选图：程序化背景 + 汉字（随机字号/颜色/旋转/水平拉伸扭曲），字符中心 = points
///
/// `chars`/`points` 均含目标字与干扰字；`target_count` 为前缀目标个数（提示词与校验仅针对目标）。
/// 干扰字降低不透明度以示区分难度（仍可干扰 OCR）。
pub fn render_click_image(
    chars: &[char],
    points: &[(f64, f64)],
    target_count: usize,
) -> RgbaImage {
    let font = FONT.get().expect("点选验证码字体未加载，请检查 captcha_font_path 配置");
    let mut bg = gen_background(IMAGE_W, IMAGE_H);
    let mut rng = rand::thread_rng();
    let palette = [
        [40u8, 60u8, 120u8],
        [140u8, 40u8, 40u8],
        [30u8, 110u8, 60u8],
        [120u8, 60u8, 140u8],
        [160u8, 90u8, 20u8],
    ];
    for (i, ch) in chars.iter().enumerate() {
        // 扭曲增强：字号范围拉大（26~46），旋转 ±30°，渲染后做水平随机拉伸（0.8~1.2）
        let size = rng.gen_range(26.0..46.0);
        let glyph = Glyph {
            id: font.glyph_id(*ch),
            scale: PxScale::from(size),
            position: ab_glyph::point(0.0, 0.0),
        };
        let Some(outlined) = font.outline_glyph(glyph) else { continue };
        let b = outlined.px_bounds();
        let (tw, th) = (b.width().ceil().max(1.0) as u32, b.height().ceil().max(1.0) as u32);
        let mut tile = RgbaImage::new(tw, th);
        let is_target = i < target_count;
        let color = palette[i % palette.len()];
        let (cr, cg, cb) = (color[0], color[1], color[2]);
        let alpha_scale = if is_target { 255.0f32 } else { 200.0 };
        outlined.draw(|x, y, a| {
            if (x as u32) < tw && (y as u32) < th {
                let alpha = (a * alpha_scale).min(255.0) as u8;
                tile.put_pixel(x as u32, y as u32, Rgba([cr, cg, cb, alpha]));
            }
        });
        // 水平拉伸扭曲（最近邻重采样）
        let tile = scale_tile_x(&tile, rng.gen_range(0.8..1.25));
        let deg = rng.gen_range(-30.0..30.0);
        rotate_and_paste(&mut bg, &tile, points[i], deg);
    }
    bg
}

/// 默认字体候选路径（程序自带思源黑体优先，相对工作目录；Windows 常见 CJK 字体兜底；生产可用 captcha_font_path 配置覆盖）
pub const DEFAULT_FONT_PATHS: &[&str] = &[
    "assets/fonts/SourceHanSansSC-Regular.otf",
    "assets/fonts/SourceHanSansSC-Bold.otf",
    "C:/Windows/Fonts/msyh.ttc",
    "C:/Windows/Fonts/simhei.ttf",
    "C:/Windows/Fonts/simsun.ttc",
    "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
];
