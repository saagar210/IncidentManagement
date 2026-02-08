-- Default services
INSERT OR IGNORE INTO services (id, name, category, default_severity, default_impact, description, is_active) VALUES
('svc-slack',      'Slack',                      'Communication',   'High',     'Critical', 'Team messaging and communication platform', 1),
('svc-zoom',       'Zoom',                       'Communication',   'High',     'Critical', 'Video conferencing platform', 1),
('svc-gworkspace', 'Google Workspace',            'Productivity',    'High',     'Critical', 'Gmail, Drive, Calendar, Docs suite', 1),
('svc-jira',       'Jira',                       'Development',     'High',     'High',     'Project and issue tracking', 1),
('svc-confluence', 'Confluence',                  'Development',     'Medium',   'Medium',   'Documentation and knowledge base', 1),
('svc-github',     'GitHub',                      'Development',     'High',     'High',     'Source code management and CI/CD', 1),
('svc-vpn',        'VPN',                         'Infrastructure',  'Critical', 'Critical', 'Remote access VPN for all employees', 1),
('svc-cloudflare', 'Cloudflare',                  'Infrastructure',  'Critical', 'Critical', 'CDN, DNS, and DDoS protection', 1),
('svc-gcp',        'Google Cloud Platform',       'Infrastructure',  'Critical', 'Critical', 'Cloud infrastructure and services', 1),
('svc-aws',        'AWS',                         'Infrastructure',  'Critical', 'Critical', 'Cloud infrastructure and services', 1),
('svc-okta',       'Okta / SSO',                  'Security',        'Critical', 'Critical', 'Identity and single sign-on provider', 1),
('svc-wifi',       'Corporate Wi-Fi',             'Infrastructure',  'High',     'High',     'Office wireless network', 1),
('svc-greenhouse', 'Greenhouse',                  'Productivity',    'High',     'Low',      'Applicant tracking and recruiting', 1),
('svc-salesforce', 'Salesforce',                  'Productivity',    'High',     'Medium',   'CRM platform', 1),
('svc-dns',        'Internal DNS',                'Infrastructure',  'Critical', 'Critical', 'Internal DNS resolution services', 1);

-- Default quarter configuration (FY27)
INSERT OR IGNORE INTO quarter_config (id, fiscal_year, quarter_number, start_date, end_date, label) VALUES
('fy27-q1', 2027, 1, '2026-02-02', '2026-04-30', 'FY27 Q1'),
('fy27-q2', 2027, 2, '2026-05-01', '2026-07-31', 'FY27 Q2'),
('fy27-q3', 2027, 3, '2026-08-01', '2026-10-31', 'FY27 Q3'),
('fy27-q4', 2027, 4, '2026-11-01', '2027-01-31', 'FY27 Q4');
