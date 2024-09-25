use std::collections::HashMap;
use std::ffi::{OsStr, OsString};
use std::ops::Sub;
use std::path;
use NodeData::*;

use http_bytes::http;

use crate::http_utils;
use crate::endpoints::{self, Endpoint};

//RouteNode: struct for each node in routing tree. holds an id (path/subpath)
//Done as a routing tree to allow for sub-routes and all that
//As in, 
//each branch node represents a route with many endpoints under it,
//leach leaf node represents an endpoint function, which takes the remaining bit of the path as input
struct RouteNode{
    data: NodeData
}
impl RouteNode{

    //new(): creates a new node for the given data
    pub fn new(data: NodeData) -> RouteNode{
        RouteNode{data}
    }

    /*
    route(): takes a path iterator, and returns the appropriate endpoint function
        parameters:
            reference to self
            mutable reference to a URL path Iterator
        returns:
            a Result:
                Ok holds a reference to a Box pointing to a handler function for the given path
                Err holds a string, giving an explanation for the error
    */
    pub fn route(&self, path: &mut path::Iter) -> Result<&Endpoint, String>{
        
        //check for what type of data this node holds:
        match &self.data{

            //if holding subnodes (tree structure),
            Branch(child_nodes) => {
                
                //get the next string in the URL path (since it's an iterator)
                let target = path.next().unwrap_or(&OsStr::new("/"));

                //get the target out from the hash map of child nodes
                match child_nodes.get(target){
                    
                    //if found, recurse into the child
                    Some(child) => {
                        return child.route(path)
                    },
                    //if not, the given path does not lead to anything! return an error
                    None => {
                        Err(format!{"looking for subpath {:?} but node not found", target})
                    }
                }
            },
            //if holding an endpoint,
            Leaf(func_box) => {
                //we found our target! return the pointer to the handler function
                return Ok(func_box);
            }
        }
    }
    
    /*
    get_node(): takes a path iterator, and returns the requested node if it exists
        parameters:
            mutable reference to self
            mutable reference to a URL path Iterator
        returns:
            a Result:
                Ok holds a mutable reference to the found node
                Err holds a string, giving an explanation for the error
    */
    pub fn find_node(&mut self, path: &mut std::path::Iter) -> Result<&mut RouteNode, String>{
        //set the target string to the next string in the URL path iterator
        let target = path.next();

        //check the target:
        match target{

            //if it exists,
            Some(str) => {

                //we must be looking for a child node
                //so, look at what data the current node holds,
                match &mut self.data{
                    
                    //if holding sub nodes,
                    Branch(child_nodes) => {

                        //get a mutable reference to the target out of the map
                        match child_nodes.get_mut(str){

                            //if found, recurse into the child
                            Some(child) => {
                                return child.find_node(path)
                            },

                            //if not, the given path does not exist!
                            None => {
                                
                                Err(format!{"looking for subpath {:?} but node not found", target})
                            }
                        }
                    },

                    //if holding an endpoint,
                    Leaf(_func_box) => {
                        //we can't search further! but theres still things in the path!
                        //return an error
                        Err(format!{"looking for subpath {:?} but ran into endpoint", target})
                    }
                }
            },

            //if the target string is empty,
            //we must be at the end of the search!
            None => {
                //return a mutable reference to self
                Ok(self)
            }
        }
        
    }
    
    /*
    add_child(): creates a new node from some data and adds it to the current node's children
        parameters:
            mutable reference to self
            URL path id for the new child, as a &str
            NodeData to give to the node
        returns:
            a mutable reference to self, to allow chaining calls
    */
    pub fn add_child(&mut self, id: &str, node_data: NodeData) -> &mut RouteNode{
        let os_id = OsString::from(id);
        match &mut self.data{
            Branch(map) => {
                map.insert(OsString::from(os_id.clone()), Box::from(RouteNode::new(node_data)));
                return self;
            },
            Leaf(_) => {
                panic!("DONT ADD CHILDREN TO AN ENDPOINT SILLY");
            }
        }
    }
    
    //similar to above but instead returns a mutable reference to the new node
    //used for adding a new Subnodes node and immediately chaining into add_child() calls on it
    pub fn add_and_select_child(&mut self, id: &str, node_data: NodeData) -> &mut RouteNode{
        let os_id = OsString::from(id);
        match &mut self.data{
            Branch(map) => {
                map.insert(OsString::from(os_id.clone()), Box::from(RouteNode::new(node_data)));
                return map.get_mut(&os_id).expect("somehow could not find the node i just added");
            },
            Leaf(_) => {
                panic!("DONT ADD CHILDREN TO AN ENDPOINT SILLY");
            }
        }
    }
    
    //get_children(): returns the node's own subnode map
    pub fn get_children(&self, id: &OsString) -> Result<&HashMap<OsString, Box<RouteNode>>, String>{
        match &self.data{
            Branch(map) => {
                Ok(&map)
            },
            Leaf(_) => {
                Err(format!("attempted to get children of an endpoint node at {:?}", id))
            }
        }
    }

}

//NodeData: data types that a node can hold: either childnodes or endpoints (functions)
enum NodeData{
    Branch(HashMap<OsString, Box<RouteNode>>),
    Leaf(Endpoint)
}

//Router: larger struct for building and holding RouteNode trees
pub struct Router{
    get: RouteNode,
    post: RouteNode,
    not_found:Box<dyn Fn() -> http::Response<Vec<u8>>>,
    bad_request: Box<dyn Fn() -> http::Response<Vec<u8>>>//TODO: error + 404 pages
}
impl Router{
    //new(): returns default Router with hard-coded routes
    pub fn new() -> Router{
        Router{get: Router::build_get_routes(), post: Router::build_post_routes(), not_found:Box::new(endpoints::index::not_found), bad_request: Box::new(endpoints::index::bad_request)}
    }

    //Router::route(): follow the path for the appropriate http method, 
    //find its endpoint, run it with the rest of the path as arguments, and return the response
    //TODO: return result instead of just crashing
    pub fn route(&self, path_iterator: &mut path::Iter, method: &str) -> Result<&Endpoint, &Box<dyn Fn() -> http::Response<Vec<u8>>>>{
        
        let tree: &RouteNode;

        //then, check what HTTP method the request used, and select the proper tree/data for it
        match method.to_lowercase().as_str(){
            
            //GET:
            "get" => {
                tree = &self.get;
            },

            //POST:
            "post" => {
                tree = &self.post;
            },
            _ => {
                return Err(&self.bad_request)
            }
        }
        
        //run RouteNode::route() on the target tree
        match tree.route(path_iterator){
                    
            //if found, we have the target handler function
            Ok(func) => {
                Ok(func)
            },

            //if not found, print the error, and return a 404 NOT FOUND
            Err(why) => {
                println!("ERROR: {}", why);
                Err(&self.not_found)
            }
        }
    }

    //build_get_routes(): builds the GET method routes
    fn build_get_routes() -> RouteNode{
        let mut tree: RouteNode = RouteNode::new(NodeData::Branch(HashMap::new()));
        
        tree.add_and_select_child("/", Branch(HashMap::new()))
        .add_child("/", Leaf(Endpoint::func(Box::new(endpoints::index::index))))
        .add_child("hello_world", Leaf(Endpoint::func(Box::new(endpoints::index::hello_world))))
        .add_child("file", Leaf(Endpoint::func(Box::new(endpoints::files::get_file))))
        .add_child("favicon.ico", Leaf(Endpoint::func(Box::new(endpoints::files::favicon))))
        ;
        
        tree
    }

    //build_post_routes(): builds the POST method routes
    fn build_post_routes() -> RouteNode{
        let mut tree: RouteNode = RouteNode::new(NodeData::Branch(HashMap::new()));
        
        tree.add_and_select_child("/", Branch(HashMap::new()))
        .add_and_select_child("users", Branch(HashMap::new()))
        .add_child("register", Leaf(Endpoint::RegisterRequest))
        .add_child("login", Leaf(Endpoint::LoginRequest))
        ;

        tree
    }
}