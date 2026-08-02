-- 文章定时发布调度任务
-- 每 5 分钟检查一次，将 publish_time <= now() 且 status != 2 的文章自动发布
INSERT INTO mxx_system_scheduler_job (job_code, job_name, cron_expression, handler, description, job_type, enabled, deleted, create_time, update_time)
SELECT 'article_publish', '文章定时发布', '0 */5 * * * *', 'article_publish', '定时检查并发布到时间的文章（publish_time <= now() 且 status != 2）', 0, 1, 0, NOW(), NOW()
WHERE NOT EXISTS (
    SELECT 1 FROM mxx_system_scheduler_job WHERE job_code = 'article_publish' AND deleted = 0
);
