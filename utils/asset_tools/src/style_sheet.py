
def set_dark_theme(app):


    DARK_THEME_STYLESHEET = """
    /* General Background */
    QWidget {
        background-color: #2e2e2e;
        color: #f0f0f0;
        font-size: 12px;
    }
/* QRadioButton*/
    QRadioButton {
        background-color: transparent;
        spacing: 5px;
    }

    QRadioButton::indicator {
        width: 15px;
        height: 15px;
        border: 1px solid #555;
        background-color: #444;
        border-radius: 7.5px;  /* Half of the width and height to get a perfect circle */
    }

    QRadioButton::indicator:checked {
        background-color: #8dc9c8;
        border-color: #444;
        image: url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAB6UlEQVQ4jX2TvYsTQRjGn5nsaiIeRs9tTgJ3gl1O9rBSm82R6Sy0EASb3J+ggu2hvfcvGERFtEju1EKncBFtrNakEo3G+EFCIGwOziyz57wWZ+J+nW8zX8/vmfed4WXIiGq16jDG1jnnDgBord0wDK+5rusltSy6cBxn0TTNdc55bVRYwtfiWTTO30XznYFWLwetdT0Mw1uu63ZjBkKIV4yxImPMnphH0Zmv4MfcCkAEffkGAIJsGXjw5gAGYw4i8ojIl1JWDADgnDud46sYFZYwKiwCBIAI0ABAABHEcghRVmj1cmj3cvb9t3kAAJ+m8ml+dV8Y+LtHhNOlXVw9F8zKnhnsCRKwRgxOzaMGx359ScNTcQJufTPSBsZukAEj8+ad4N/nGXtn1Lxp3bt4/UKQAvjDDUBHzAgoj7ewQF5zlgERbcq2gc8Dnq4zAc+FfZwI3oOINmcGUso6EXl3nuUzao6Py9tbICJPSlmPvYFSaq0zYP7G80I8iwhc3n6Kw2HfV0qtpR7RdV1PKVV52Tb8241D2Jl+NQGGnmBl/AQLE89XSlWiPZFDJLrdbr9UKr346ZtXXn84mD9p/cbjj6dwxn+EI+p7CgYSzTQN27aLlmU1ot04HA4veZ7nZ+n3DSFETQhR+5/mD71OMPR5GmfFAAAAAElFTkSuQmCC") !important; 
    }
/* dropdown*/
    QComboBox::drop-down {
        border: 1px solid #555;
    }
    QComboBox::drop-down:hover {
        border: 1px solid #777;
    }

/* Buttons */
    QPushButton {
        background-color: #555;
        border: 1px solid #777;
        padding: 4px;
        border-radius: 3px;
    }

    QPushButton:disabled {
        background-color: #3e3e3e;
        border: 1px solid #555;
        color: #777;
    }

    QPushButton:hover {
        background-color: #777;
    }

    QPushButton:pressed {
        background-color: #999;
    }

/* QSplitter handle styling */
    QSplitter::handle {
        background: #5a5a5a;
    }
    QSplitter::handle:horizontal {
        width: 5px;
        image: url(vertical-line.png); /* Optionally use an image for the handle */
    }
    QSplitter::handle:vertical {
        height: 5px;
        image: url(horizontal-line.png); /* Optionally use an image for the handle */
    }

/* ComboBox */
    QComboBox {
        border: 1px solid #777;
        border-radius: 3px;
        padding: 4px;
        background-color: #555;
    }

    QComboBox:disabled {
        border: 1px solid #555;
        color: #777;
    }

    QComboBox:hover {
        border-color: #999;
    }

/* Qcheckboxes */
    QCheckBox {
        padding: 4px;
    }

    QCheckBox::indicator {
        width: 16px;
        height: 16px;
        border: 1px solid #777;
        background-color: #555;
        border-radius: 2px;
    }
    
    /*//image: url(:/qt-project.org/styles/commonstyle/images/standardbutton-ok-16.png); */
    QCheckBox::indicator:checked {
        background-color: #8dc9c8;
        border: 1px solid #999;
        image: url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAAAdpJREFUOI3FkL+L02Acxp80yZtfpZE0FGqFUpu0uQxSiOB2FgcHOTgnZ52cRFDO0X9AuL/B9RwEp0NEnHQ5Dg7BvVh/RMkbeJuWNs2P1+WQ41p6mz7j93n48OEL/OMIG1vHcWqe5z1d19m2PSSEHCqKsrMR0u/3DzzPe3L21m63m7IsfwHAJUnaP9tVzgNEUdzjnD/2ff8eAPi+Tyil+1mW+aeTXxsNTi1eOI7zzXXdbdu29wRB4AC4KIojQsjWhYDBYHDJcZyTTqfzVdd1BoATQk5M07y6YrwOEIbhwrbtRlEUu4qiKACODcO4E8fx+Px25QcA0O12G1EU3c6yDKIowrKsy7VabXc4HEoXGvR6PZsx9opSui3L8m9VVRec80ZZljtJklyr1+sf4zierDXwfd+aTqcHURQNDcP4qev6A0LI80qlkgNAURR38zz/5LrujRWDVqtVT9P0TRiGN03THFmW9XA8Hr+jlB5ZltUsy/I6AHDOzbIsQ8bYh7+AIAj05XL5djabdarV6pEois8mk8l3TdOuqKranM/nx5qmVQRB2AIgC4LwmTF2CAASACFJkvtpmr7O8/w9pfQHgAWAFEAGIA+CQGaMPZIk6WWWZbfKshyte/7/yR+CNq/HGxFTVwAAAABJRU5ErkJggg==") !important; 
    }

    QCheckBox::indicator:unchecked {
        background-color: #555;
        border: 1px solid #777;
    }

    QCheckBox::indicator:hover {
        border: 1px solid #999;
    }

    QCheckBox::indicator:disabled {
        border: 1px solid #555;
        background-color: #3e3e3e;
    }

/* ClickableComboBox styling */

    ClickableComboBox, ClickableComboBox QListView, ClickableComboBox QLineEdit {
        background-color: #2e2e2e;
        color: #ffffff;
        border: 1px solid #5a5a5a;
        padding: 5px;
    }
        ClickableComboBox::drop-down {
        width: 20px;
        border: none;
    }

   ClickableComboBox QCheckBox::indicator:checked {
        background-color: #8dc9c8;
        border: 1px solid #999;
        image: url("data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAABAAAAAQCAYAAAAf8/9hAAAACXBIWXMAAA7EAAAOxAGVKw4bAAAAGXRFWHRTb2Z0d2FyZQB3d3cuaW5rc2NhcGUub3Jnm+48GgAAAdpJREFUOI3FkL+L02Acxp80yZtfpZE0FGqFUpu0uQxSiOB2FgcHOTgnZ52cRFDO0X9AuL/B9RwEp0NEnHQ5Dg7BvVh/RMkbeJuWNs2P1+WQ41p6mz7j93n48OEL/OMIG1vHcWqe5z1d19m2PSSEHCqKsrMR0u/3DzzPe3L21m63m7IsfwHAJUnaP9tVzgNEUdzjnD/2ff8eAPi+Tyil+1mW+aeTXxsNTi1eOI7zzXXdbdu29wRB4AC4KIojQsjWhYDBYHDJcZyTTqfzVdd1BoATQk5M07y6YrwOEIbhwrbtRlEUu4qiKACODcO4E8fx+Px25QcA0O12G1EU3c6yDKIowrKsy7VabXc4HEoXGvR6PZsx9opSui3L8m9VVRec80ZZljtJklyr1+sf4zierDXwfd+aTqcHURQNDcP4qev6A0LI80qlkgNAURR38zz/5LrujRWDVqtVT9P0TRiGN03THFmW9XA8Hr+jlB5ZltUsy/I6AHDOzbIsQ8bYh7+AIAj05XL5djabdarV6pEois8mk8l3TdOuqKranM/nx5qmVQRB2AIgC4LwmTF2CAASACFJkvtpmr7O8/w9pfQHgAWAFEAGIA+CQGaMPZIk6WWWZbfKshyte/7/yR+CNq/HGxFTVwAAAABJRU5ErkJggg==") !important; 
    }

    ClickableComboBox::down-arrow {
        image: url(down-arrow.png); /* Replace with your arrow image */
    }

    ClickableComboBox QAbstractItemView {
        background-color: #2e2e2e;
        color: #ffffff;
        selection-background-color: #5a5a5a;
    }

    ClickableComboBox QAbstractItemView::item {
        height: 20px;
    }

    ClickableComboBox QAbstractItemView::item:hover {
        background-color: #3e3e3e;
        color: #ffffff;
    }

    ClickableComboBox QAbstractItemView::item:selected {
    background-color: #4a4a88;
    }

    
    ClickableComboBox QAbstractItemView::indicator {
        width: 13px;
        height: 13px;
    }

    ClickableComboBox QAbstractItemView::indicator:unchecked {
     border: 1px solid #999;
    }

    ClickableComboBox QAbstractItemView::indicator:checked {
        background-color: #8dc9c8;
        border: 1px solid #999;
    }


/* SpinBox and DoubleSpinBox */
    QSpinBox, QDoubleSpinBox {
        border: 1px solid #777;
        border-radius: 3px;
        background-color: #555;
        padding: 2px;
    }

    QSpinBox:disabled, QDoubleSpinBox:disabled {
        border: 1px solid #555;
        color: #777;
    }

/* Scroll Bar */
    QScrollBar {
        border: 1px solid #555;
        background-color: #2e2e2e;
    }

    QScrollBar::handle {
        background-color: #555;
    }

    QScrollBar::handle:hover {
        background-color: #777;
    }

/* GraphicsView */
    QGraphicsView {
        border: 1px solid #555;
        background-color: #555;
    }

/* Tab Widget */
    QTabWidget::pane {
        border: 1px solid #555;
    }

    QTabBar::tab {
        background: #555;
        border: 1px solid #777;
        padding: 4px;
        border-top-left-radius: 3px;
        border-top-right-radius: 3px;
        min-width: 100px;
    }

    QTabBar::tab:selected {
        background: #777;
    }

    QTabBar::tab:!selected {
        background: #555;
    }

    QTabBar::tab:hover {
        background: #666;
    }

    QTabBar::tab:disabled {
        color: #777;
    }

/* ToolBox */
    QToolBox::tab {
        background-color: #555;
        border: 1px solid #777;
        border-radius: 3px;
        padding: 4px;
    }

    QToolBox::tab:selected {
        background-color: #666;
    }

/* Gray out disabled QToolBox items */
    QToolBox::tab:disabled {
    color: #777;
    background-color: #222;
    }

/* Tooltips */
    QToolTip {
        background-color: #555;
        color: #eee;
        border: 1px solid #777;
        padding: 4px;
        border-radius: 3px;
    }

/* Dialogs */
    QDialog {
        background-color: #2e2e2e;
    }

/* Menu */
    QMenu {
        background-color: #2e2e2e;
        border: 1px solid #777;
    }

    QMenu::item {
        padding: 4px 20px 4px 20px;
    }

    QMenu::item:selected {
        background-color: #777;
    }

/* Styling for the scroll bar */

    QScrollBar:vertical {
        border: 1px solid #555;
        background: #333;
        width: 15px;
    }

    QScrollBar::handle:vertical {
        background: #555;
        min-height: 20px;
    }

    QScrollBar::add-line:vertical, QScrollBar::sub-line:vertical {
        border: none;
        background: none;
    }

/* QLineEdit, QTextEdit*/
    QLineEdit, QTextEdit{
        background-color: #444;
        border: 1px solid #555;
        border-radius: 3px;
    }
    """



    app.setStyleSheet(DARK_THEME_STYLESHEET)