import os
import sys
import unittest

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from wxauto_agent import select_wechat_main_hwnd


class SelectWeChatMainHwndTests(unittest.TestCase):
    def test_prefers_wechat_exe_path(self):
        windows = [
            (101, "OtherClass", "微信"),
            (202, "WeChatMainWnd", "微信"),
        ]
        path_by_hwnd = {
            101: r"C:\Program Files\Other\Other.exe",
            202: r"C:\Program Files\Tencent\WeChat\WeChat.exe",
        }
        self.assertEqual(select_wechat_main_hwnd(windows, path_by_hwnd), 202)

    def test_falls_back_to_title(self):
        windows = [
            (303, "OtherClass", "微信"),
        ]
        self.assertEqual(select_wechat_main_hwnd(windows, {}), 303)

    def test_matches_class_name_when_title_missing(self):
        windows = [
            (404, "WeChatMainWndForPC", ""),
        ]
        self.assertEqual(select_wechat_main_hwnd(windows, {}), 404)

    def test_returns_none_when_no_candidate(self):
        windows = [
            (505, "OtherClass", "Notepad"),
        ]
        self.assertIsNone(select_wechat_main_hwnd(windows, {}))


if __name__ == "__main__":
    unittest.main()
