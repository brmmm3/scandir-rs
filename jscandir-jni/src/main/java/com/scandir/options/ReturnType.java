package com.scandir.options;

/**
 * Return type for scan results.
 * Matches the C cscandir_return_type enum.
 */
public enum ReturnType {
    BASE(0),
    EXT(1);

    private final int value;

    ReturnType(int value) {
        this.value = value;
    }

    public int getValue() {
        return value;
    }

    public static ReturnType fromValue(int value) {
        for (ReturnType t : values()) {
            if (t.value == value) return t;
        }
        throw new IllegalArgumentException("Invalid ReturnType value: " + value);
    }
}