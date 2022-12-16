from flask import Flask
import webbrowser
from action.tags_reader import TagsReader

app = Flask(__name__)
 
@app.route("/")
def index():
    return "Congratulations, it's a web app!"

@app.route("/analyze")
def doAnalyze():
    TagsReader.readToGoogleSheet(dir = "/Users/duongdoan/Downloads", googleSheetKey='1qlCRJ5wuAf7q5J37gwJa0MvMx0Vpgrm3lqzAD_VwyE4', prepare=False)
    return "Done"

if __name__ == "__main__":
  webbrowser.open_new('127.0.0.1:8000')
  app.run(host="127.0.0.1", port=8000, debug=True)
    

