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

            // get css
            const css = document.querySelector("code.language-style");

            if (document.getElementById("custom-styles")) {
                document.getElementById("custom-styles").remove();
            }

            if (css !== null) {
                const stylesheet = document.createElement("style");

                stylesheet.id = "custom-styles";
                stylesheet.innerHTML = css.innerText;

                css.remove();
                document.body.appendChild(stylesheet);
            }

            // handle modification blocks
            function mod_attr(css_property) {
                for (const element of Array.from(
                    document.querySelectorAll(`[data-${css_property}]`),
                )) {
                    element.style.setProperty(
                        css_property,
                        element.getAttribute(`data-${css_property}`),
                    );
                }
            }

            mod_attr("color");
            mod_attr("font-family");

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
