{% if ask_mode %}Analyze the ticket and repo context. Do not change code. Create
`SOLUTION_SUMMARY.md`.

Ticket: {{ ticket.key }} - {{ ticket.title }}
{% if has_knowledge_base %}
Knowledge base: Read `{{ knowledge_base_dir }}/` in the workspace before analysis.
{% endif %}
{% else %}Fix the ticket with minimal, compatible changes and tests.

Ticket: {{ ticket.key }} - {{ ticket.title }}
{% if has_knowledge_base %}
Knowledge base: Read `{{ knowledge_base_dir }}/` in the workspace before changes.
{% endif %}
Build: `{{ main_build }}`
{% if test_compile %}Test compile: `{{ test_compile }}`
{% endif %}Tests: `{{ test_run }}`

Create `SOLUTION_SUMMARY.md` after completion.

{% if additional_prompt %}Additional requirements: {{ additional_prompt }}

{% endif %}{% endif %}
