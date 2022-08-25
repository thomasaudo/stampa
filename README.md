# Profile photo manager

I developed this project to discover web server development with Rust and Actix. 
The project is clearly not finished, there is also a lot of refactoring / testing to do.

The objective is to develop a server that allows to easily manage users' profile pictures.  

Thanks to an API key, for a given project, the administrator can store the profile pictures of his users or generate one if the user does not have a picture.

He can then retrieve the pictures via a URL, he can specify parameters (size, width) and also ask for the profile picture to be cropped on the head (thanks to AWS Rekognition).