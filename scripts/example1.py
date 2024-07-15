#!/usr/bin/env python3
from emit_config import remap

lmeta = 'KEY_LEFTMETA'
lctrl = 'KEY_LEFTCTRL'
lshift = 'KEY_LEFTSHIFT'
lalt = 'KEY_LEFTALT'
rmeta = 'KEY_RIGHTMETA'
rctrl = 'KEY_RIGHTCTRL'
ralt = 'KEY_RIGHTALT'
rshift = 'KEY_RIGHTSHIFT'
mod = {lmeta, lctrl, lshift, lalt, rmeta, rctrl, ralt, rshift}

latin = ['KEY_' + chr(65 + i) for i in range(26)]
num = ['KEY_' + str(i) for i in range(10)]
fn = ['KEY_F' + str(i) for i in range(1, 25)]
kpnum = ['KEY_KP' + str(i) for i in range(10)]

space = 'KEY_SPACE'
left, right, down, up = 'KEY_LEFT', 'KEY_RIGHT', 'KEY_DOWN', 'KEY_UP'
home, end = 'KEY_HOME', 'KEY_END'
lbr, rbr = 'KEY_LEFTBRACE', 'KEY_RIGHTBRACE'

caps = 'KEY_CAPSLOCK'

all = {
    *latin, *num, *fn, *kpnum, *mod, caps, 'KEY_NUMLOCK', 'KEY_SCROLLLOCK',
    'KEY_ESC', 'KEY_MINUS', 'KEY_EQUAL', 'KEY_BACKSPACE', 'KEY_TAB',
    lbr, rbr, 'KEY_ENTER',
    'KEY_SEMICOLON', 'KEY_APOSTROPHE', 'KEY_GRAVE', 'KEY_BACKSLASH',
    'KEY_COMMA', 'KEY_DOT', 'KEY_SLASH',
    'KEY_KPASTERISK', space,
    'KEY_KPMINUS', 'KEY_KPPLUS', 'KEY_KPDOT', 'KEY_ZENKAKUHANKAKU', 'KEY_102ND',
    'KEY_RO', 'KEY_KATAKANA', 'KEY_HIRAGANA', 'KEY_HENKAN', 'KEY_KATAKANAHIRAGANA', 'KEY_MUHENKAN', 'KEY_KPJPCOMMA', 'KEY_KPENTER',
    'KEY_KPSLASH', 'KEY_SYSRQ', 'KEY_LINEFEED',
    home, end, left, right, down, up, 'KEY_PAGEUP', 'KEY_PAGEDOWN',
    'KEY_INSERT', 'KEY_DELETE',
    'KEY_MACRO', 'KEY_MUTE', 'KEY_VOLUMEDOWN', 'KEY_VOLUMEUP', 'KEY_POWER', 'KEY_KPEQUAL', 'KEY_KPPLUSMINUS', 'KEY_PAUSE', 'KEY_SCALE',
    'KEY_KPCOMMA', 'KEY_HANGEUL', 'KEY_HANJA', 'KEY_YEN', 'KEY_COMPOSE',
}

remap([], lctrl, lmeta, {lmeta: lctrl})
remap([], lctrl, latin, {lmeta: lctrl})
remap([], lctrl, num, {lmeta: lctrl})

remap(lmeta, lalt, left, {lmeta: [], left: home})
remap(lmeta, lalt, right, {lmeta: [], right: end})
remap([], [], [home, end], {lmeta: lctrl})

remap([], lctrl, ['KEY_TAB', 'KEY_GRAVE'], {lmeta: lmeta})
remap(lmeta, [], space, {lmeta: [], space: lmeta})
remap(lmeta, [], lbr, {lmeta: lalt, lbr: left})
remap(lmeta, [], rbr, {lmeta: lalt, rbr: right})
remap([], [], ['KEY_MINUS', 'KEY_EQUAL'], {lmeta: lctrl})

remap([], [*(mod - {lalt, ralt})], lalt, {lalt: []})
remap(lalt, lmeta, [left, right, 'KEY_BACKSPACE', 'KEY_DELETE'], {lalt: lctrl})
remap(lalt, [], [*(all - {left, right, 'KEY_BACKSPACE', 'KEY_DELETE'})], {lalt: lalt})

remap([], [*mod], caps, {caps: 'KEY_HANGEUL'})

remap([lmeta, lctrl], [], 'KEY_T', {lmeta: lalt})
remap([lmeta, lctrl], [], 'KEY_Q', {lctrl: [], lmeta: lmeta, 'KEY_Q': 'KEY_L'})
