import sys
import time
import subprocess

CHAR_MAP = {
    'a': 'KEY_A', 'b': 'KEY_B', 'c': 'KEY_C', 'd': 'KEY_D', 'e': 'KEY_E',
    'f': 'KEY_F', 'g': 'KEY_G', 'h': 'KEY_H', 'i': 'KEY_I', 'j': 'KEY_J',
    'k': 'KEY_K', 'l': 'KEY_L', 'm': 'KEY_M', 'n': 'KEY_N', 'o': 'KEY_O',
    'p': 'KEY_P', 'q': 'KEY_Q', 'r': 'KEY_R', 's': 'KEY_S', 't': 'KEY_T',
    'u': 'KEY_U', 'v': 'KEY_V', 'w': 'KEY_W', 'x': 'KEY_X', 'y': 'KEY_Y',
    'z': 'KEY_Z',
    '0': 'KEY_0', '1': 'KEY_1', '2': 'KEY_2', '3': 'KEY_3', '4': 'KEY_4',
    '5': 'KEY_5', '6': 'KEY_6', '7': 'KEY_7', '8': 'KEY_8', '9': 'KEY_9',
    ' ': 'KEY_SPACE',
    '\n': 'KEY_ENTER',
    '-': 'KEY_MINUS',
    '=': 'KEY_EQUAL',
    '/': 'KEY_SLASH',
    '.': 'KEY_DOT',
    ',': 'KEY_COMMA',
    ';': 'KEY_SEMICOLON',
    '\'': 'KEY_APOSTROPHE',
    '[': 'KEY_LEFTBRACE',
    ']': 'KEY_RIGHTBRACE',
    '\\': 'KEY_BACKSLASH',
    ':': ['KEY_LEFTSHIFT', 'KEY_SEMICOLON'],
    '"': ['KEY_LEFTSHIFT', 'KEY_APOSTROPHE'],
    '_': ['KEY_LEFTSHIFT', 'KEY_MINUS'],
    '+': ['KEY_LEFTSHIFT', 'KEY_EQUAL'],
    '|': ['KEY_LEFTSHIFT', 'KEY_BACKSLASH'],
    '>': ['KEY_LEFTSHIFT', 'KEY_DOT'],
    '<': ['KEY_LEFTSHIFT', 'KEY_COMMA'],
}

def type_text(vm_name, text):
    for char in text:
        lower_char = char.lower()
        if char.isupper():
            keys = ['KEY_LEFTSHIFT', f'KEY_{char}']
        elif char in CHAR_MAP:
            keys = CHAR_MAP[char]
            if not isinstance(keys, list):
                keys = [keys]
        else:
            print(f"Unknown char: {char}", file=sys.stderr)
            continue
        
        cmd = ['virsh', '--connect', 'qemu:///system', 'send-key', vm_name] + keys
        subprocess.run(cmd, check=True)
        time.sleep(0.04)

if __name__ == '__main__':
    vm = sys.argv[1]
    text = sys.argv[2]
    type_text(vm, text)
