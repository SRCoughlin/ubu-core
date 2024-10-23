We can use Eclipse IDE for building dynamic web project with JSPs and use Tomcat to run it. Please read [Java Web Applications](/community/tutorials/java-web-application-tutorial-for-beginners#first-web-app-servlet) tutorial to learn how can we easily create JSPs in Eclipse and run it in tomcat. A simple JSP example page example is: `home.jsp`

```
<%@ page language="java" contentType="text/html; charset=US-ASCII"
    pageEncoding="US-ASCII"%>
<!DOCTYPE html PUBLIC "-//W3C//DTD HTML 4.01 Transitional//EN" "https://www.w3.org/TR/html4/loose.dtd">
<html>
<head>
<meta http-equiv="Content-Type" content="text/html; charset=US-ASCII">
<title>First JSP</title>
</head>
<%@ page import="java.util.Date" %>
<body>
<h3>Hi Pankaj</h3><br>
<strong>Current Time is</strong>: <%=new Date() %>

</body>
</html>
```

If you have a simple JSP that uses only JRE classes, we are not required to put it as WAR file. Just create a directory in the tomcat webapps folder and place your JSP file in the newly created directory. For example, if your JSP is located at apache-`tomcat/webapps/test/home.jsp`, then you can access it in browser with URL `https://localhost:8080/test/home.jsp`. If your host and port is different, then you need to make changes in URL accordingly.
