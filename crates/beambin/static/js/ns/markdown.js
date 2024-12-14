(() => {
    const markdown = reg_ns("markdown", ["app", "bundled_env"]);

    markdown.define(
        "fix_markdown",
        function ({ app, bundled_env }, root_id) {
            const theme = document.querySelector(`#${root_id} theme`);

            if (theme !== null) {
                if (theme.innerText === "dark") {
                    document.documentElement.classList.add("dark");
                } else {
                    document.documentElement.classList.remove("dark");
                }

                // update icon
                app.update_theme_icon();
            }

            // get js
            const bundled = document.querySelector("code.language-worker");

            if (bundled !== null) {
                if (bundled_env.workers && bundled_env.workers.length > 0) {
                    // make sure we don't leave the old workers running
                    for (worker of bundled_env.workers) {
                        console.info("terminated old worker");
                        worker.terminate();
                    }
                }

                bundled_env.enter_env(bundled.innerText);
                bundled.remove();
            }

            // handle modification blocks
            for (const script of Array.from(
                document.querySelectorAll(`#${root_id} script[type="env/mod"]`),
            )) {
                try {
                    const mods = JSON.parse(script.innerHTML);
                    let element = script.previousSibling;

                    // find something that isn't useless
                    // (anything but #text)
                    while (element.nodeName === "#text") {
                        element = element.previousSibling;
                    }

                    // update attributes
                    for (const entry of Object.entries(mods)) {
                        element.setAttribute(entry[0], entry[1]);
                    }

                    element.setAttribute("data-env-modified", "true");
                    script.remove();
                } catch (err) {
                    console.error("MOD:", err);
                    continue;
                }
            }

            // escape all code blocks
            for (const block of Array.from(
                document.querySelectorAll("#tab\\:preview pre code"),
            )) {
                block.innerHTML = block.innerHTML
                    .replaceAll("<", "&lt;")
                    .replaceAll(">", "&gt;");
            }

            // highlight
            hljs.highlightAll();
        },
        ["string"],
    );

    markdown.define("use_template", function ({ $ }, slug) {
        $.dialog = document.getElementById("template_dialog");
        $.dialog.showModal();

        document
            .getElementById("template_form")
            .addEventListener("submit", async (e) => {
                e.preventDefault();

                const res = await (
                    await fetch("/api/v1/posts/clone", {
                        method: "POST",
                        headers: {
                            "Content-Type": "application/json",
                        },
                        body: JSON.stringify({
                            slug: e.target.slug.value,
                            password: e.target.password.value,
                            source: slug,
                        }),
                    })
                ).json();

                if (res.success === false) {
                    trigger("app:shout", ["error", res.message]);

                    $.dialog.close();
                } else {
                    window.location.href = `/${res.payload[1].slug}?ANNC=${res.payload[0]}`;
                }
            });
    });
})();
