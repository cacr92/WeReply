import os
import sys
import unittest


ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
VENDOR = os.path.join(ROOT, "vendor", "wxauto")
if VENDOR not in sys.path:
    sys.path.insert(0, VENDOR)

from wxauto.ui.layout import (  # noqa: E402
    LayoutLabels,
    _find_navigation_container,
    _find_session_container,
    _looks_like_session_container,
)


class FakeControl:
    def __init__(self, control_type, name="", children=None):
        self.ControlTypeName = control_type
        self.Name = name
        self._children = children or []
        self._parent = None
        for child in self._children:
            child._parent = self

    def GetChildren(self):
        return self._children

    def GetParentControl(self):
        return self._parent


class LayoutFallbackTests(unittest.TestCase):
    def setUp(self):
        self.labels = LayoutLabels(
            session_list_names={"会话"},
            session_search_names={"搜索"},
            message_list_names={"消息"},
            send_button_names={"发送"},
            navigation_button_names={"聊天", "通讯录"},
        )

    def test_navigation_fallback_uses_button_count(self):
        nav_container = FakeControl(
            "PaneControl",
            children=[
                FakeControl("ButtonControl"),
                FakeControl("ButtonControl"),
                FakeControl("ButtonControl"),
            ],
        )
        session_container = FakeControl(
            "PaneControl",
            children=[
                FakeControl("EditControl", name="搜索"),
                FakeControl("ListControl"),
            ],
        )
        chat_container = FakeControl(
            "PaneControl",
            children=[
                FakeControl("EditControl"),
                FakeControl("ButtonControl", name="发送"),
            ],
        )
        root = FakeControl(
            "WindowControl",
            children=[nav_container, session_container, chat_container],
        )
        found = _find_navigation_container(root, self.labels, 6)
        self.assertIs(found, nav_container)

    def test_session_fallback_uses_type_matching(self):
        session_container = FakeControl(
            "PaneControl",
            children=[
                FakeControl("EditControl", name="搜索"),
                FakeControl("ListControl"),
            ],
        )
        root = FakeControl("WindowControl", children=[session_container])
        found = _find_session_container(root, self.labels, 6)
        self.assertIs(found, session_container)

    def test_session_container_rejects_send_button(self):
        container = FakeControl(
            "PaneControl",
            children=[
                FakeControl("EditControl"),
                FakeControl("ListControl"),
                FakeControl("ButtonControl", name="发送"),
            ],
        )
        self.assertFalse(_looks_like_session_container(container, self.labels))


if __name__ == "__main__":
    unittest.main()
