package com.scandir.error;

/**
 * Error thrown when a scandir operation fails.
 */
public class JScandirError extends RuntimeException {
    private final int code;
    private final String message;

    public JScandirError(int code, String message) {
        super("cscandir error " + code + ": " + message);
        this.code = code;
        this.message = message;
    }

    public JScandirError(int code, String message, Throwable cause) {
        super("cscandir error " + code + ": " + message, cause);
        this.code = code;
        this.message = message;
    }

    public JScandirError(String message) {
        this(0, message);
    }

    public JScandirError(String message, Throwable cause) {
        this(0, message, cause);
    }

    public int getCode() { return code; }
    public String getMessage() { return message; }

    public static JScandirError fromNative(int code, String message) {
        return new JScandirError(code, message);
    }

    public static JScandirError invalidArgument(String message) {
        return new JScandirError(1, message);
    }

    public static JScandirError invalidUtf8(String message) {
        return new JScandirError(2, message);
    }

    public static JScandirError scanError(String message) {
        return new JScandirError(3, message);
    }

    public static JScandirError nulByte(String message) {
        return new JScandirError(4, message);
    }
}