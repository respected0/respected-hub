// Vite turns an imported SVG into its URL (small ones become inline data URLs, which the app's CSP allows).
declare module '*.svg' { const url: string; export default url; }
