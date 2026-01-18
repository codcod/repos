# Repos Fix - {{ mode_title }}

Ticket: {{ ticket.key }} - {{ ticket.title }}
Platform: {{ platform_name }}

{% if ask_mode %}
ASK mode: analyze only. Do not change code. Produce `SOLUTION_SUMMARY.md`.
{% else %}
Make minimal, compatible changes. Add or update tests as needed.

Build: `{{ main_build }}`
{% if test_compile %}Test compile: `{{ test_compile }}`
{% endif %}Tests: `{{ test_run }}`
{% endif %}
