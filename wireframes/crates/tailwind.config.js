const defaultTheme = require('tailwindcss/defaultTheme');
module.exports = {
  content: ["./src/**/*.{html,js}"],
  theme: {
    extend: {
	    fontFamily: { sans: ['Fira',...defaultTheme.fontFamily.sans], }
    },
  },
  plugins: [
      require('@tailwindcss/forms'),
  ]
}
