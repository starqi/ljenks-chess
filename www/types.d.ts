// Needed for TS checking of PNG imports
declare module '*.png' {
    const value: string;
    export default value;
}
