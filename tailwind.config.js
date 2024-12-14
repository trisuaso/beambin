/** @type {import('tailwindcss').Config} */
module.exports = {
    content: ["./crates/beambin/templates/**/*.html"],
    darkMode: "class",
    theme: {
        extend: {
            colors: {
                brand: "rgb(254 187 0)",
                "brand-low": "rgb(229 168 0)",
            },
            animation: {
                "fade-in": "fadein 0.25s ease-in-out 1 running",
            },
            keyframes: {
                fadein: {
                    "0%": { opacity: "0%" },
                    "100%": { opacity: "100%" },
                },
            },
        },
    },
    plugins: [],
};
