import os
import sys
import unittest

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", "vendor", "wxauto"))
if ROOT not in sys.path:
    sys.path.insert(0, ROOT)

from wxauto.ui.layout import LayoutLabels, discover_main_controls


class FakeControl:
    def __init__(self, name="", control_type="", children=None):
        self.Name = name
        self.ControlTypeName = control_type
        self._children = children or []
        self._parent = None
        for child in self._children:
            child._parent = self

    def GetChildren(self):
        return list(self._children)

    def GetParentControl(self):
        return self._parent

    def Walk(self, includeTop=False, maxDepth=0xFFFFFFFF):
        def walk(node, depth):
            if includeTop or depth > 0:
                yield node, depth
            if depth >= maxDepth:
                return
            for child in node.GetChildren():
                yield from walk(child, depth + 1)

        return walk(self, 0)


def default_labels():
    return LayoutLabels(
        session_list_names={"会话", "折叠的群聊"},
        session_search_names={"搜索"},
        message_list_names={"消息"},
        send_button_names={"发送", "发送(S)"},
        navigation_button_names={"聊天", "通讯录", "收藏"},
    )


class LayoutDiscoveryTests(unittest.TestCase):
    def test_discovers_main_controls_from_tree(self):
        nav = FakeControl(
            control_type="PaneControl",
            children=[
                FakeControl(name="聊天", control_type="ButtonControl"),
                FakeControl(name="通讯录", control_type="ButtonControl"),
                FakeControl(name="收藏", control_type="ButtonControl"),
            ],
        )
        session_list = FakeControl(name="会话", control_type="ListControl")
        search_box = FakeControl(name="搜索", control_type="EditControl")
        session = FakeControl(control_type="PaneControl", children=[search_box, session_list])

        msg_list = FakeControl(name="消息", control_type="ListControl")
        edit_box = FakeControl(name="", control_type="EditControl")
        send_btn = FakeControl(name="发送(S)", control_type="ButtonControl")
        chat = FakeControl(control_type="PaneControl", children=[msg_list, edit_box, send_btn])

        root = FakeControl(control_type="WindowControl", children=[nav, session, chat])

        controls = discover_main_controls(root, default_labels())

        self.assertIs(controls.navigation, nav)
        self.assertIs(controls.session, session)
        self.assertIs(controls.chat, chat)

    def test_falls_back_to_send_button_when_message_list_missing(self):
        nav = FakeControl(
            control_type="PaneControl",
            children=[
                FakeControl(name="聊天", control_type="ButtonControl"),
                FakeControl(name="通讯录", control_type="ButtonControl"),
                FakeControl(name="收藏", control_type="ButtonControl"),
            ],
        )
        session_list = FakeControl(name="会话", control_type="ListControl")
        search_box = FakeControl(name="搜索", control_type="EditControl")
        session = FakeControl(control_type="PaneControl", children=[search_box, session_list])

        edit_box = FakeControl(name="", control_type="EditControl")
        send_btn = FakeControl(name="发送", control_type="ButtonControl")
        chat = FakeControl(control_type="PaneControl", children=[edit_box, send_btn])

        root = FakeControl(control_type="WindowControl", children=[nav, session, chat])

        labels = default_labels()
        labels = LayoutLabels(
            session_list_names=labels.session_list_names,
            session_search_names=labels.session_search_names,
            message_list_names={"不存在"},
            send_button_names=labels.send_button_names,
            navigation_button_names=labels.navigation_button_names,
        )

        controls = discover_main_controls(root, labels)

        self.assertIs(controls.chat, chat)

    def test_accepts_document_and_table_controls_for_new_layouts(self):
        nav = FakeControl(
            control_type="PaneControl",
            children=[
                FakeControl(name="聊天", control_type="ButtonControl"),
                FakeControl(name="通讯录", control_type="ButtonControl"),
                FakeControl(name="收藏", control_type="ButtonControl"),
            ],
        )
        session_list = FakeControl(name="会话", control_type="TableControl")
        search_box = FakeControl(name="搜索", control_type="DocumentControl")
        session = FakeControl(control_type="PaneControl", children=[search_box, session_list])

        msg_list = FakeControl(name="消息", control_type="DataGridControl")
        edit_box = FakeControl(name="", control_type="DocumentControl")
        send_btn = FakeControl(name="发送", control_type="ButtonControl")
        chat = FakeControl(control_type="PaneControl", children=[msg_list, edit_box, send_btn])

        root = FakeControl(control_type="WindowControl", children=[nav, session, chat])

        controls = discover_main_controls(root, default_labels())

        self.assertIs(controls.navigation, nav)
        self.assertIs(controls.session, session)
        self.assertIs(controls.chat, chat)


if __name__ == "__main__":
    unittest.main()
