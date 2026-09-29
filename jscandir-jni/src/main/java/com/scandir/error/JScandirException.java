package com.scandir.error;

/**
 * Exception thrown when a scandir operation fails.
 */
public class JScandirException extends Exception {
    private final int code;
    private final String message;

    public JScandirException(int code, String message) {
        super("cscandir error " + code + ": " + message);
        this.code = code;
        this.message = message;
    }

    public JScandirException(int code, String message, Throwable cause) {
        super("cscandir error " + code + ": " + message, cause);
        this.code = code;
        this.message = message;
    }

    public JScandirException(String message) {
        this(0, message);
    }

    public JScandirException(String message, Throwable cause) {
        this(0, message, cause);
    }

    public int getCode() { return code; }
    public String getMessage() { return message; }

    public static JScandirException fromNative(int code, String message) {
        return new JScandirException(code, message);
    }

    public static JScandirException invalidArgument(String message) {
        return new JScandirException(1, message);
    }

    public static JScandirException invalidUtf8(String message) {
        return new JScandirException(2, message);
    }

    public JScandirException scanError(String message) {
        return new JScandirException(3, message);
    }

    public static JScandirException nulByte(String message) {
        return new JScandirException(4, message);
    }
}