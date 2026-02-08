# {{ platform_emoji }} {{ ticket.key }} - {{ ticket.title }}

Use `mission-context.json` to understand the repo and constraints. Make minimal,
compatible changes and keep code style consistent.

- **Platform**: {{ platform_name }}
- **Languages**: {{ languages }}
- **Frameworks**: {{ frameworks }}

{{ platform_guidelines }}

{% if has_knowledge_base %}
## Domain knowledge base
Relevant platform/journey docs are available in `{{ knowledge_base_dir }}/` in
this workspace. Read these before making changes.

{% if knowledge_base_files and knowledge_base_files | length > 0 %}
Available files:
{% for file in knowledge_base_files %}
- `{{ knowledge_base_dir }}/{{ file }}`
{% endfor %}
{% endif %}

{% if knowledge_base_content %}
### Inlined highlights (selected)
{{ knowledge_base_content }}
{% endif %}
{% endif %}

## Build and test
- **Build**: `{{ main_build }}`
{% if test_compile %}- **Test compile**: `{{ test_compile }}`
{% endif %}- **Tests**: `{{ test_run }}`

{% if additional_prompt %}
## Additional requirements
{{ additional_prompt }}
{% endif %}
