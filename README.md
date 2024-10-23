UBU mark 2

--------------
FEATURES
--------------
GUI:
-events like gCalendar
-tasks like Todoist
-update estimated left
-establish dependencies & priorities
-label tasks
-show geography/location

External data feed:
X-GitHub (notification as task)
-gMail (email as task)
-Todoist
-gCalendar
-Wanikani

External sensor feed:
-Goggle Fit
-Garmin

Sensors:
-Firefox tabs
-GPS/location
-Accelerometer


--------------
INTERFACES
--------------
UX:
-REST
	-NPM client web page by default
	-XDG?

D2D:
-Bluetooth (specific)
-i2p (specific)
-Tor Onion Service (REST)
-WiFi direct (reseach)


--------------
MODEL
--------------
Task:
-type:
	-has Events
	-has sub-Tasks
	-neither (idea/aspiration)
-Stats: {pending, active, inactive, complete, deleted}
-Event[]
-SubTasks: Task[]
-Parent: Task
-EstimatedLeft: time
-Category
-Label[]
-Start
-Due
-Location[] (can be none or one-of-many)
-Resource[] (can be subset-of-many)
-OnComplete()
---
-ToDoist ID
-gMail ID
-GitHub ID

Event:
-Start
-End
-Task
-Location?
---
-gCalendar ID

Category enum:
-Business...

Label:
-type:
	-Person
	-Category
	-Idea
	
Service-specific:
GitHub:
-AccessToken
-NextPoll
-Last-Modified


--------------
STRATEGY
--------------
-DB
-Print tasks
-daemon: https://github.com/vasilakisfil/hello.service
-IMAP: read/write
	- https://github.com/jonhoo/rust-imap
	- order by thread/subject, then date
	- thread is super-task, email is task
	- thread is inactive, never closed
	- super-task is prioritized by oldest unread email/active sub-task
-gCalendar API: pull
	- https://github.com/erikh/gcal
-Todoist API: pull
	- https://developer.todoist.com/rest/v2/#overview
	- https://developer.todoist.com/sync/v9/#summary-of-contents
-Synchronize tasks
-gMail API: pull and create Task
-GitHub API: pull and create Task
	- https://docs.github.com/en/apps/creating-github-apps/about-creating-github-apps/about-creating-github-apps
	- https://docs.github.com/en/rest/activity/notifications?apiVersion=2022-11-28
	- https://docs.github.com/en/rest/overview/authenticating-to-the-rest-api?apiVersion=2022-11-28
-Buy server
-Create OAuth2 system
	-lang: ?
-gCalendar OAuth2


--------------
FUTURE
--------------
-Desktop notifications: https://github.com/hoodie/notify-rust


--------------
EXAMPLE
-------------
IMAP loop:
-read message
-search for existing task by subject else create new single
	-if single, create new super-task and reassign old as sub-task
	-add new sub-task with HTML for display with last sub-task as pre-dependency
-mark as read


NOTES: 2024-07-26:
UI:
1) Initialize
2) Operate [control]
3) Configure: New
4) Configure: Update

UX flow:
1) Initialize=>setup
2:MAIN) Perform task
	-update estimated left
3) 2=>Review calendar
4) 2=>Complete
5) 2=>INSTEAD (new task/review tasks)
6) 2=>Split with INSTEAD (new task/review tasks)
7) 2=>Setup
8) 2=>[Task==Configure]=>Configure: Update

Background:
-Create/update calendar/plans
-Enforce/query dependency DAG [Task==Configure,HIGH]
-Query priority weights (DAG?) [Task==Configure,MEDIUM]
-Query tags [Task==Configure,LOW]


Tag intelligence:
-ML/AI based on training

Dependency intelligence:
-Use tags
-Also use tag intelligence
-If from other source [calendar,task tracker] use start/due dates as weak tag

Priority intelligence:
-Use tags
-Also use tag intelligence
-value is independent of dependencies, but still use dependency intelligence
-If from other source [calendar,task tracker] use meta as strong tag
