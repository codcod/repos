# {{ platform_emoji }} {{ ticket.key }} - {{ ticket.title }}

Use `mission-context.json` to understand the repo and constraints. Make minimal,
compatible changes and keep code style consistent.

- **Platform**: {{ platform_name }}
- **Languages**: {{ languages }}
- **Frameworks**: {{ frameworks }}

{{ platform_guidelines }}

## Build and test
- **Build**: `{{ main_build }}`
{% if test_compile %}- **Test compile**: `{{ test_compile }}`
{% endif %}- **Tests**: `{{ test_run }}`

{% if additional_prompt %}
## Additional requirements
{{ additional_prompt }}
{% endif %}
