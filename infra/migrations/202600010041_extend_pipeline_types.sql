-- Extend pipeline_type to include diff_review
ALTER TABLE cicd_pipelines
    DROP CONSTRAINT cicd_pipelines_pipeline_type_check;

ALTER TABLE cicd_pipelines
    ADD CONSTRAINT cicd_pipelines_pipeline_type_check
    CHECK (pipeline_type IN (
        'build', 'lint', 'secret_scan', 'sast',
        'dependency_scan', 'image_scan', 'diff_review'
    ));
