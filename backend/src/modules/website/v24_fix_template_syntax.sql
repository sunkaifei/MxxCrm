-- v24_fix_template_syntax.sql
-- 修复模板中 minijinja 不支持的 || 语法，改为 or 关键字
-- 使用 dollar-quoting 避免单引号冲突

-- type_id=7 产品列表页: {{ p.image_url || ' }}/static/images/no-image.png"  →  {{ p.image_url or '/static/images/no-image.png' }}
UPDATE mxx_template_user_data
SET temptext = replace(temptext,
    $old${{ p.image_url || ' }}/static/images/no-image.png"$old$,
    $new${{ p.image_url or '/static/images/no-image.png' }}"$new$)
WHERE type_id = 7 AND temptext LIKE '%p.image_url ||%';

-- type_id=8 产品详情页: {{ site.copyright || site.site_name }}  →  {{ site.copyright or site.site_name }}
UPDATE mxx_template_user_data
SET temptext = replace(temptext,
    $old${{ site.copyright || site.site_name }}$old$,
    $new${{ site.copyright or site.site_name }}$new$)
WHERE type_id = 8 AND temptext LIKE '%site.copyright || site.site_name%';

-- type_id=14 公共页头: {{ site.company_name || site.site_name }}  →  {{ site.company_name or site.site_name }}
UPDATE mxx_template_user_data
SET temptext = replace(temptext,
    $old${{ site.company_name || site.site_name }}$old$,
    $new${{ site.company_name or site.site_name }}$new$)
WHERE type_id = 14 AND temptext LIKE '%site.company_name || site.site_name%';

-- type_id=15 公共页脚: {{ site.copyright || site.site_name }}  →  {{ site.copyright or site.site_name }}
UPDATE mxx_template_user_data
SET temptext = replace(temptext,
    $old${{ site.copyright || site.site_name }}$old$,
    $new${{ site.copyright or site.site_name }}$new$)
WHERE type_id = 15 AND temptext LIKE '%site.copyright || site.site_name%';

-- 验证: 检查是否还有 || 残留
SELECT type_id, name, position('||' in temptext) as remaining_pos
FROM mxx_template_user_data
WHERE temptext LIKE '%||%'
ORDER BY type_id;
