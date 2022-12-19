from PyQt6.QtWidgets import QMainWindow, QApplication, QPushButton, QFileDialog, QHBoxLayout, QWidget, QTabWidget,QLineEdit, QLabel, QVBoxLayout
from PyQt6.QtCore import QSize, Qt

import sys


class Main(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Music Tags Reader")
        self.setFixedSize(QSize(800, 600))
        widgetAnalyze = QWidget()
        widgetAnalyze.setFixedSize(QSize(800, 500))
        widgetUpdate = QWidget()
        tabs = QTabWidget()
        #tabs.setTabPosition(QTabWidget.West)
        tabs.addTab(widgetAnalyze, 'Read Tags')
        tabs.addTab(widgetUpdate, 'Write Tags')

        self.setCentralWidget(tabs)
        

        layoutAnalyze = QVBoxLayout()
        widgetAnalyze.setLayout(layoutAnalyze)

        layoutSelectDir = QHBoxLayout()
        layoutAnalyze.addLayout(layoutSelectDir)

        self.textDir = QLineEdit(self)
        labelDir = QLabel(self)
        labelDir.setText("Enter Music Dir")
        layoutSelectDir.addWidget(labelDir)
        #self.textDir.setFixedSize(QSize(300, 35))
        layoutSelectDir.addWidget(self.textDir)
        btnSelectDir = QPushButton(self)
        #btnSelectDir.setFixedSize(QSize(150, 35))
        btnSelectDir.setText("Select")
        btnSelectDir.clicked.connect(lambda: self.select_dir_dialog())
        layoutSelectDir.addWidget(btnSelectDir)

        layoutSelectOutputFile = QHBoxLayout()
        layoutAnalyze.addLayout(layoutSelectOutputFile)

        self.textOutputFile = QLineEdit(self)
        labelOutputFile = QLabel(self)
        labelOutputFile.setText("Enter Output File")
        layoutSelectOutputFile.addWidget(labelOutputFile)
        #self.textDir.setFixedSize(QSize(300, 35))
        layoutSelectOutputFile.addWidget(self.textOutputFile)
        btnSelectFile = QPushButton(self)
        #btnSelectDir.setFixedSize(QSize(150, 35))
        btnSelectFile.setText("Select")
        btnSelectFile.clicked.connect(lambda: self.select_file_dialog())
        layoutSelectOutputFile.addWidget(btnSelectFile)

        layoutAction = QVBoxLayout()
        layoutAnalyze.addLayout(layoutAction)
        self.labelStatus = QLabel(self)
        layoutSelectOutputFile.addWidget(self.labelStatus)

    def select_dir_dialog(self):
        fname = QFileDialog.getExistingDirectory(self, "Select Directory",
                                                "/home");
        self.textDir.setText(fname)


    def select_file_dialog(self):
        fname = QFileDialog.getOpenFileName(
            self,
            "Select Output File",
            "${HOME}",
            "All Files (*)",
        )
        self.textOutputFile.setText(fname)
        
    
if __name__ == "__main__":
    app = QApplication(sys.argv)
    main_gui = Main()
    main_gui.show()
    sys.exit(app.exec())