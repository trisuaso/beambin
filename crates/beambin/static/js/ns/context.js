// paste metadata editor
(() => {
    const context = reg_ns("context");

    context.define(
        "context_editor",
        function ({ $ }, bind_to, paste_url, context) {
            $.context = context;

            globalThis.update_metadata_value = (name, value) => {
                $.context[name] = value;
                console.log(context);
            };

            // ...
            if (Object.entries($.context).length == 0) {
                bind_to.innerHTML = `<div class="card secondary round">
                    <span>No metadata options available.</span>
                </div>`;
            }

            // render
            for (const field of Object.entries($.context)) {
                if (
                    globalThis._app_base.starstraw === false &&
                    field[0] === "owner"
                ) {
                    continue;
                }

                if (field[0] === "template") {
                    const paste_is_template = field[1] === "@";
                    const paste_source =
                        paste_is_template === false ? field[1] : "";

                    if (!paste_is_template && !paste_source) {
                        globalThis.mark_as_template = () => {
                            $.context.template = "@";

                            // rerender all
                            bind_to.innerHTML = "";
                            $.context_editor(bind_to, paste_url, context);
                            return;
                        };

                        bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                            <label for="field_input:${field[0]}">${field[0]}</label>
                            <button class=\"theme:primary round\" onclick=\"globalThis.mark_as_template()\" type=\"button\">Mark as Template</button>
                        </div>`;
                    } else if (paste_is_template) {
                        globalThis.mark_as_not_template = () => {
                            $.context.template = "";

                            // rerender all
                            bind_to.innerHTML = "";
                            $.context_editor(bind_to, paste_url, context);
                            return;
                        };

                        bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                            <label for="field_input:${field[0]}">${field[0]}</label>
                            <button class=\"theme:primary round\" onclick=\"globalThis.mark_as_not_template()\" type=\"button\">Unmark as Template</button>
                        </div>`;
                    } else if (paste_source) {
                        bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                            <label for="field_input:${field[0]}">${field[0]}</label>
                            <a class=\"button !text-sky-800 dark:!text-sky-300 round\" href=\"/${paste_source}\" title=\"${paste_source}\">View Source</button>
                        </div>`;
                    }

                    continue;
                }

                bind_to.innerHTML += `<div class="card secondary round flex justify-between items-center gap-2" style="flex-wrap: wrap;" id="field:${field[0]}">
                    <label for="field_input:${field[0]}">${field[0]}</label>
                    <input
                      id="field_input:${field[0]}"
                      type="text"
                      value="${field[1].replace('"', '\\"')}"
                      onchange="globalThis.update_metadata_value('${field[0]}', event.target.value)"
                      style="width: max-content"
                      ${field[0] === "owner" ? "disabled" : ""}
                    />
                </div>`;
            }
        },
    );

    context.define("submit_hook", function ({ $ }, slug) {
        document
            .getElementById("submit_form")
            .addEventListener("submit", async (e) => {
                e.preventDefault();

                const res = await (
                    await fetch(`/api/v1/posts/${slug}/context`, {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            password: e.target.password.value,
                            context: $.context,
                        }),
                    })
                ).json();

                if (res.success === false) {
                    window.location.href = `?ANNC=${res.message}&ANNC_TYPE=error`;
                } else {
                    window.location.href = `?ANNC=${res.message}`;
                }
            });
    });
})();
