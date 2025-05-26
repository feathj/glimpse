Glimpse
=======
A swiss army knife for AI image processing and computer vision.

What is it?
-----------
Glimpse is a rust application for AI image processing and computer vision. One night as I was
probably trying to go to sleep, I had an idea. "AI models are getting really good at computer
vision tasks, what general purpose tasks could I abstract out and use them for on local images?
Can I store metadata associated with those tasks in the image exif data? That would make it
extremely portable and easy to use." So I started working on Glimpse.

What can I do with it?
----------------------
I was presented with two really compelling workflows almost as soon as I had the idea. The first
was the ability to automatically tag images with known people in them. Given a sample image,
the name of the person, and a directory of images, Glimpse can automatically tag all images with
that person in them. This could be extremely useful for organizing a large collection of images
based on the people in them.

The second workflow was the ability to describe images, and group images based on their
descriptions. Given a directory of images, and a prompt, glimpse can generate a text description
of the image, and save that description in the exif data of the image. Furthermore, glimpse can
save the description in an embeddings format that can be used to group similar images together.

Because a prompt can be provided to generate the description, there are many potential workflows
that could be built on top of this. For example, you could ask glimpse to describe all of the
images in a directory, which gets saved to the exif data. You could then take one image of a
person skiing, and then ask glimpse to find all of the images of people skiing in the
directory.

How do I build and use it?
--------------------------
Glimpse is a standard rust application, so you can build it with cargo. You will need to have
rust installed on your system, or you can use a prebuilt binary from the releases page.

Once you have the binary, you can run `glimpse --help` to see the available commands and options.

Currently, glimpse requires AWS credentials with access to both bedrock and regkognition or OpenAI
credentials. Note that OpenAI does not support facial regognition, so if you are using OpenAI, face
tagging will not be available.

Some sample commands
--------------------
Generate a description of all images in a directory:
```sh
glimpse \
--action tag-description \
--files /path/to/images 
```

Now find images based on a description:
```sh
glimpse \
--action find \
--description "A person skiing" \
--files /path/to/images
```

Or find images based on an existing image with a description:
```sh
glimpse \
--action find-similar \
--reference-file /path/to/reference/image \
--files /path/to/images
```

Tag all images in a directory with a known person:
Note: this relies on the aws "rekognition" service, so you will need to have an aws account with
the rekognition service enabled and credentials available.
```sh
glimpse \
--action tag-person \
--person-name "John" \
--reference-file /path/to/reference/image
```

For best results, run `tag-person` with multiple reference images of the same person.
If the person is already tagged in an image, glimpse will not tag the person again.

Find all images with a known person in them:
```sh
glimpse \
--action find-person \
--person-name "John" \
--files /path/to/images
```

Given a list of tags, tag all images with AI classification based on generated description:
```sh
glimpse \
--action tag \
--tags "tag1,tag2,tag3" \
--files /path/to/images
```

Sort by tag. This will move all images with a given tag to a directory with the tag name. If
an image has multiple tags, it will utilize the first one found.
```sh
glimpse \
--action sort-by-tag \
--files /path/to/images \
--output-directory /path/to/output \
```