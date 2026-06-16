//!
//! Copyright (c) 2024-2999 北京心月狐科技有限公司 All rights reserved.
//!
//! https://www.mxxshop.com
//!
//! Licensed 并不是自由软件，未经许可不能去掉 MxxShop 相关版权
//!
//! 版权所有，侵权必究！
//!

use crate::core::errors::error::Result;
use crate::core::web::tags::common_tags::{filter_html, none_default, to_json_filter};
use crate::core::web::tags::format_time_tags::format_time;
use minijinja::{path_loader, Environment, Value};


pub fn get_template(name: &str, ctx: Value) -> Result<String> {
    let mut env = Environment::new();
    env.set_auto_escape_callback(|_| minijinja::AutoEscape::Html);
    env.set_loader(path_loader("templates"));
    // 注册自定义过滤器
    env.add_filter("to_json", to_json_filter);
    env.add_filter("default", none_default);
    env.add_function("format_time", format_time);
    env.add_function("filter_html",filter_html);
    //env.add_function("lang", lang_function);
    let tpl = env.get_template(name)?;
    //log::info!("===========tpl=========={:?}",tpl);
    Ok(tpl.render(ctx).unwrap_or_default())
}


pub fn get_template_a(template_content: &str, ctx: Value) -> Result<String> {
    let mut env = Environment::new();
    // 注册自定义过滤器
    env.add_filter("to_json", to_json_filter);
    env.add_filter("default", none_default);
    env.add_function("format_time", format_time);
    env.add_function("filter_html", filter_html);
    
    // 添加模板字符串
    let r = env.render_str(template_content,ctx)?;
    Ok(r)
}


//pub fn get_website()



// 定义全局标签插入函数
use std::collections::HashMap;

#[allow(dead_code)]
fn render_template(template: &str, context: &HashMap<&str, String>) -> String {
    let mut rendered = template.to_string();
    for (key, value) in context {
        let placeholder = format!("{{{{ {} }}}}", key); // 假设使用 {{ key }} 作为占位符
        rendered = rendered.replace(&placeholder, value);
    }
    rendered
}




