(() => {
    const app = reg_ns("app");

    // env
    app.DEBOUNCE = [];

    // ...
    app.define("try_use", function (_, ns_name, callback) {
        // attempt to get existing namespace
        if (globalThis._app_base.ns_store[`$${ns_name}`]) {
            return callback(globalThis._app_base.ns_store[`$${ns_name}`]);
        }

        // otherwise, call normal use
        use(ns_name, callback);
    });

    app.define("debounce", function ({ $ }, name) {
        return new Promise((resolve, reject) => {
            if ($.DEBOUNCE.includes(name)) {
                return reject();
            }

            $.DEBOUNCE.push(name);

            setTimeout(() => {
                delete $.DEBOUNCE[$.DEBOUNCE.indexOf(name)];
            }, 1000);

            return resolve();
        });
    });

    app.define("rel_date", function (_, date) {
        // stolen and slightly modified because js dates suck
        const diff = (new Date().getTime() - date.getTime()) / 1000;
        const day_diff = Math.floor(diff / 86400);

        if (Number.isNaN(day_diff) || day_diff < 0 || day_diff >= 31) {
            return;
        }

        return (
            (day_diff === 0 &&
                ((diff < 60 && "just now") ||
                    (diff < 120 && "1 minute ago") ||
                    (diff < 3600 && Math.floor(diff / 60) + " minutes ago") ||
                    (diff < 7200 && "1 hour ago") ||
                    (diff < 86400 &&
                        Math.floor(diff / 3600) + " hours ago"))) ||
            (day_diff === 1 && "Yesterday") ||
            (day_diff < 7 && day_diff + " days ago") ||
            (day_diff < 31 && Math.ceil(day_diff / 7) + " weeks ago")
        );
    });

    app.define("clean_date_codes", function ({ $ }) {
        for (const element of Array.from(document.querySelectorAll(".date"))) {
            if (element.getAttribute("data-unix")) {
                // this allows us to run the function twice on the same page
                // without errors from already rendered dates
                element.innerText = element.getAttribute("data-unix");
            }

            element.setAttribute("data-unix", element.innerText);
            const then = new Date(Number.parseInt(element.innerText));

            if (Number.isNaN(element.innerText)) {
                continue;
            }

            element.setAttribute("title", then.toLocaleString());

            let pretty = $.rel_date(then);

            if (screen.width < 900 && pretty !== undefined) {
                // shorten dates even more for mobile
                pretty = pretty
                    .replaceAll(" minutes ago", "m")
                    .replaceAll(" minute ago", "m")
                    .replaceAll(" hours ago", "h")
                    .replaceAll(" hour ago", "h")
                    .replaceAll(" days ago", "d")
                    .replaceAll(" day ago", "d")
                    .replaceAll(" weeks ago", "w")
                    .replaceAll(" week ago", "w")
                    .replaceAll(" months ago", "m")
                    .replaceAll(" month ago", "m")
                    .replaceAll(" years ago", "y")
                    .replaceAll(" year ago", "y");
            }

            element.innerText =
                pretty === undefined ? then.toLocaleDateString() : pretty;

            element.style.display = "inline-block";
        }
    });

    app.define("copy_text", function ({ $ }, text) {
        navigator.clipboard.writeText(text);
        $.toast("success", "Copied!");
    });

    // hooks
    app.define("hook.scroll", function (_, scroll_element, track_element) {
        const goals = [150, 250, 500, 1000];

        track_element.setAttribute("data-scroll", "0");
        scroll_element.addEventListener("scroll", (e) => {
            track_element.setAttribute("data-scroll", scroll_element.scrollTop);

            for (const goal of goals) {
                const name = `data-scroll-${goal}`;
                if (scroll_element.scrollTop >= goal) {
                    track_element.setAttribute(name, "true");
                } else {
                    track_element.removeAttribute(name);
                }
            }
        });
    });

    app.define("hook.dropdown", function (_, event) {
        event.stopImmediatePropagation();
        let target = event.target;

        while (!target.matches(".dropdown")) {
            target = target.parentElement;
        }

        // close all others
        for (const dropdown of Array.from(
            document.querySelectorAll(".inner[open]"),
        )) {
            dropdown.removeAttribute("open");
        }

        // open
        setTimeout(() => {
            for (const dropdown of Array.from(
                target.querySelectorAll(".inner"),
            )) {
                // check y
                const box = target.getBoundingClientRect();

                let parent = dropdown.parentElement;

                while (!parent.matches("html, .window")) {
                    parent = parent.parentElement;
                }

                let parent_height = parent.getBoundingClientRect().y;

                if (parent.nodeName === "HTML") {
                    parent_height = window.screen.height;
                }

                const scroll = window.scrollY;
                const height = parent_height;
                const y = box.y + scroll;

                if (y > height - scroll - 300) {
                    dropdown.classList.add("top");
                } else {
                    dropdown.classList.remove("top");
                }

                // open
                dropdown.toggleAttribute("open");

                if (dropdown.getAttribute("open")) {
                    dropdown.removeAttribute("aria-hidden");
                } else {
                    dropdown.setAttribute("aria-hidden", "true");
                }
            }
        }, 5);
    });

    app.define("hook.dropdown.init", function (_, bind_to) {
        for (const dropdown of Array.from(
            document.querySelectorAll(".inner"),
        )) {
            dropdown.setAttribute("aria-hidden", "true");
        }

        bind_to.addEventListener("click", (event) => {
            if (
                event.target.matches(".dropdown") ||
                event.target.matches("[exclude=dropdown]")
            ) {
                return;
            }

            for (const dropdown of Array.from(
                document.querySelectorAll(".inner[open]"),
            )) {
                dropdown.removeAttribute("open");
            }
        });
    });

    app.define("hook.alt", function (_) {
        for (const element of Array.from(
            document.querySelectorAll("img") || [],
        )) {
            if (element.getAttribute("alt") && !element.getAttribute("title")) {
                element.setAttribute("title", element.getAttribute("alt"));
            }
        }
    });

    // web api replacements
    app.define("prompt", function (_, msg) {
        const dialog = document.getElementById("web_api_prompt");
        document.getElementById("web_api_prompt:msg").innerText = msg;

        return new Promise((resolve, _) => {
            globalThis.web_api_prompt_submit = (value) => {
                dialog.close();
                return resolve(value);
            };

            dialog.showModal();
        });
    });

    app.define("confirm", function (_, msg) {
        const dialog = document.getElementById("web_api_confirm");
        document.getElementById("web_api_confirm:msg").innerText = msg;

        return new Promise((resolve, _) => {
            globalThis.web_api_confirm_submit = (value) => {
                dialog.close();
                return resolve(value);
            };

            dialog.showModal();
        });
    });

    // adomonition
    app.define("shout", function (_, type, content) {
        if (document.getElementById("admonition")) {
            // there can only be one
            document.getElementById("admonition").remove();
        }

        const element = document.createElement("div");

        element.id = "admonition";
        element.classList.add("markdown-alert");
        element.classList.add(type);

        element.innerHTML = `<span class="markdown-alert-title">${content
            .replaceAll("<", "&lt")
            .replaceAll(">", "&gt;")}</span>`;

        if (document.querySelector("#admonition_zone")) {
            document.querySelector("#admonition_zone").prepend(element);
            return;
        }

        document.querySelector("article").prepend(element);
    });

    // shout from query params
    const search = new URLSearchParams(window.location.search);

    if (search.get("ANNC")) {
        // get defaults
        // we'll always use the value given in a query param over the page-set value
        const secret_type = search.get("ANNC_TYPE")
            ? search.get("ANNC_TYPE")
            : globalThis._app_base.annc.type;

        // ...
        app.shout(secret_type, search.get("ANNC"));
    }

    // theme
    globalThis.sun_icon = document.getElementById("theme_icon_sun");
    globalThis.moon_icon = document.getElementById("theme_icon_moon");

    app.define("update_theme_icon", function () {
        if (document.documentElement.classList.contains("dark")) {
            globalThis.sun_icon.style.display = "none";
            globalThis.moon_icon.style.display = "flex";
        } else {
            globalThis.sun_icon.style.display = "flex";
            globalThis.moon_icon.style.display = "none";
        }
    });

    app.update_theme_icon(); // initial update
    app.define("toggle_theme", function () {
        if (
            window.PASTE_USES_CUSTOM_THEME &&
            window.localStorage.getItem("se:user.ForceClientTheme") !== "true"
        ) {
            return;
        }

        const current = window.localStorage.getItem("theme");

        if (current === "dark") {
            /* set light */
            document.documentElement.classList.remove("dark");
            window.localStorage.setItem("theme", "light");
        } else {
            /* set dark */
            document.documentElement.classList.add("dark");
            window.localStorage.setItem("theme", "dark");
        }

        app.update_theme_icon();
    });

    // link filter
    app.define("link_filter", function (_) {
        for (const anchor of Array.from(document.querySelectorAll("a"))) {
            if (anchor.href.length === 0) {
                continue;
            }

            const url = new URL(anchor.href);
            if (
                anchor.href.startsWith("/") ||
                anchor.href.startsWith("javascript:") ||
                url.origin === window.location.origin
            ) {
                continue;
            }

            anchor.addEventListener("click", (e) => {
                e.preventDefault();
                document.getElementById("link_filter_url").innerText =
                    anchor.href;
                document.getElementById("link_filter_continue").href =
                    anchor.href;
                document.getElementById("link_filter").showModal();
            });
        }
    });
})();
