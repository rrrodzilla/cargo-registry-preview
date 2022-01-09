const defaultTheme = require('tailwindcss/defaultTheme');
module.exports = {
    content: ["./src/**/*.{html,js}"],
    theme: {
        extend: {
            fontFamily: {
                sans: ['Fira Sans', ...defaultTheme.fontFamily.sans],
                mono: ['Fira Mono', ...defaultTheme.fontFamily.mono],
            },
            maxWidth: {
                '960': '960px',
            },
            height: {
                '80': '80px',
                '13': '3.35rem',
            },
            fontSize: {
                'logo': '24px',
            },
            padding: {
                '38': '9.5rem',
            },
            gridTemplateRows: {
                'search-info': 'auto',
            },
            gridTemplateColumns: {
                'crate-info': 'minmax(0,7fr) minmax(0,3fr)',
                'search-info': 'auto 1fr auto',
            },
        },
    },
    plugins: [
        require('@tailwindcss/forms'),
    ]
}
