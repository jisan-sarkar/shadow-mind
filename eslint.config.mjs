import parser from "@typescript-eslint/parser";

export default [
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: { parser, parserOptions: { ecmaFeatures: { jsx: true } } },
    rules: {
      eqeqeq: ["error", "always"],
      "no-debugger": "error",
      "no-dupe-keys": "error",
      "no-unreachable": "error",
    },
  },
];
