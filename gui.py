from PyQt6.QtWidgets import QMainWindow, QApplication, QPushButton, QFileDialog, QHBoxLayout, QWidget, QTabWidget
import sys


class Main(QMainWindow):
    def __init__(self):
        super().__init__()
        self.setWindowTitle("Music Tags Reader")

        widgetAnalyze = QWidget()
        widgetUpdate = QWidget()
        tabs = QTabWidget()
        #tabs.setTabPosition(QTabWidget.West)
        tabs.addTab(widgetAnalyze, 'Read Tags')
        tabs.addTab(widgetUpdate, 'Write Tags')

        self.setCentralWidget(tabs)
        

        layoutAnalyze = QHBoxLayout()
        widgetAnalyze.setLayout(layoutAnalyze)
        


        btnSelectDir = QPushButton(self)
        btnSelectDir.setText("Select Music Directory")
        btnSelectDir.clicked.connect(lambda: self.select_dir_dialog())
        layoutAnalyze.addWidget(btnSelectDir)

    def select_dir_dialog(self):
        fname = QFileDialog.getExistingDirectory(self, "Select Directory",
                                                "/home");
        print(fname)


    def select_file_dialog(self):
        fname = QFileDialog.getOpenFileName(
            self,
            "Select Output File",
            "${HOME}",
            "All Files (*)",
        )
        print(fname)
        
    
if __name__ == "__main__":
    app = QApplication(sys.argv)
    main_gui = Main()
    main_gui.show()
    sys.exit(app.exec())